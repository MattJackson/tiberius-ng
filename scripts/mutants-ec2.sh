#!/usr/bin/env bash
# Run cargo-mutants sharded across ephemeral EC2 spot instances.
#
# Each instance clones this repo at a given git ref, runs one shard
# (`cargo mutants --shard i/N`), uploads its `mutants.out` to S3, and
# self-terminates. The driver polls S3, downloads every shard's results, and
# prints the combined surviving-mutant ("missed") list.
#
# Usage:
#   scripts/mutants-ec2.sh run [REF] [N] [INSTANCE_TYPE]
#     REF            git ref to test (default: current HEAD sha)
#     N              number of shards / instances (default: 8)
#     INSTANCE_TYPE  EC2 instance type (default: c6i.2xlarge)
#
# Requires: aws cli (configured), a repo reachable by `git clone` (public).
# Cost: N spot instances for the duration of the run (minutes), auto-terminated.
set -euo pipefail

REGION="${AWS_REGION:-us-east-1}"
REPO_URL="${MUTANTS_REPO_URL:-https://github.com/MattJackson/tiberius-ng.git}"
cmd="${1:-run}"
REF="${2:-$(git rev-parse HEAD)}"
N="${3:-8}"
ITYPE="${4:-c6i.2xlarge}"
TAG="tiberius-mutants"
BUCKET="tiberius-mutants-${RANDOM}${RANDOM}"

log() { printf '\033[36m[mutants-ec2]\033[0m %s\n' "$*" >&2; }

latest_al2023_ami() {
  aws ssm get-parameters --region "$REGION" \
    --names /aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-x86_64 \
    --query 'Parameters[0].Value' --output text
}

user_data() {
  local shard="$1"
  cat <<EOF | base64
#!/bin/bash
set -x
exec > /var/log/mutants.log 2>&1
S3="s3://$BUCKET/shard-$shard"
finish() { aws s3 cp /var/log/mutants.log "\$S3/log" || true; aws s3 cp - "\$S3/done" <<< "done" || true; shutdown -h now; }
trap finish EXIT
# System build deps for `--features all` (TLS backends + Kerberos/GSSAPI).
dnf install -y git gcc gcc-c++ make cmake openssl-devel pkgconf perl krb5-devel awscli
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
export CARGO_HOME=/root/.cargo RUSTUP_HOME=/root/.rustup
export PATH=/root/.cargo/bin:\$PATH
aws s3 cp - "\$S3/started" <<< "started"
cargo install cargo-mutants --version '^27' --locked
cd /root
git clone --depth 1 --branch "\${REF_BRANCH:-dev}" "$REPO_URL" repo || git clone "$REPO_URL" repo
cd repo
git checkout "$REF"
cargo mutants --features all --shard "$shard/$N" --in-place -j "\$(nproc)" --output out || true
aws s3 cp --recursive out/mutants.out "\$S3/" || true
EOF
}

case "$cmd" in
run)
  log "region=$REGION ref=$REF shards=$N type=$ITYPE bucket=$BUCKET"
  AMI="$(latest_al2023_ami)"
  log "AMI=$AMI"

  aws s3 mb "s3://$BUCKET" --region "$REGION" >&2

  # Minimal instance role so instances can write results to the bucket.
  ROLE="${TAG}-role"; PROFILE="${TAG}-profile"
  if ! aws iam get-role --role-name "$ROLE" >/dev/null 2>&1; then
    aws iam create-role --role-name "$ROLE" --assume-role-policy-document \
      '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"ec2.amazonaws.com"},"Action":"sts:AssumeRole"}]}' >&2
    aws iam attach-role-policy --role-name "$ROLE" \
      --policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess >&2
    aws iam create-instance-profile --instance-profile-name "$PROFILE" >&2
    aws iam add-role-to-instance-profile --instance-profile-name "$PROFILE" --role-name "$ROLE" >&2
    sleep 15 # allow the instance profile to propagate
  fi

  ids=()
  for shard in $(seq 0 $((N - 1))); do
    id=$(REF_BRANCH="$(git rev-parse --abbrev-ref HEAD)" aws ec2 run-instances \
      --region "$REGION" --image-id "$AMI" --instance-type "$ITYPE" \
      --instance-market-options '{"MarketType":"spot"}' \
      --iam-instance-profile "Name=$PROFILE" \
      --instance-initiated-shutdown-behavior terminate \
      --block-device-mappings '[{"DeviceName":"/dev/xvda","Ebs":{"VolumeSize":30}}]' \
      --user-data "$(user_data "$shard")" \
      --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$TAG-$shard}]" \
      --query 'Instances[0].InstanceId' --output text)
    log "shard $shard -> $id"
    ids+=("$id")
  done

  log "waiting for shards to finish (writing done markers to s3://$BUCKET)..."
  for i in $(seq 1 300); do
    done=$(aws s3 ls "s3://$BUCKET/" --recursive | grep -c '/done' || true)
    log "  $done/$N shards done"
    [ "$done" -ge "$N" ] && break
    sleep 30
  done

  out="mutants-ec2-results"; mkdir -p "$out"
  aws s3 cp --recursive "s3://$BUCKET/" "$out/" >&2
  echo "=== combined surviving (missed) mutants ==="
  cat "$out"/shard-*/missed.txt 2>/dev/null | sort -u
  echo "=== totals ==="
  for k in caught missed timeout unviable; do
    echo "$k: $(cat "$out"/shard-*/$k.txt 2>/dev/null | grep -cve '^[[:space:]]*$' || echo 0)"
  done
  log "results in ./$out ; bucket s3://$BUCKET (run 'down' to clean up)"
  echo "$BUCKET" > .mutants-ec2-bucket
  ;;
down)
  b="$(cat .mutants-ec2-bucket 2>/dev/null || true)"
  [ -n "$b" ] && aws s3 rb "s3://$b" --force >&2 || true
  aws ec2 describe-instances --region "$REGION" \
    --filters "Name=tag:Name,Values=${TAG}-*" "Name=instance-state-name,Values=running,pending" \
    --query 'Reservations[].Instances[].InstanceId' --output text | tr '\t' '\n' | while read -r id; do
    [ -n "$id" ] && aws ec2 terminate-instances --region "$REGION" --instance-ids "$id" >&2 || true
  done
  log "cleanup requested"
  ;;
*)
  echo "usage: $0 {run|down} [ref] [n] [type]" >&2; exit 1;;
esac
