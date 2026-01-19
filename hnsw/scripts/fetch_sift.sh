#!/usr/bin/env bash
set -euo pipefail

BASE_URL="ftp://ftp.irisa.fr/local/texmex/corpus"
ARCHIVE="sift.tar.gz"
DATA_DIR="data/sift"

mkdir -p "${DATA_DIR}"
TMP_DIR=$(mktemp -d)

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE}"

echo "Downloading ${ARCHIVE} from ${BASE_URL}..."
curl -fL "${BASE_URL}/${ARCHIVE}" -o "${ARCHIVE_PATH}"

for FILE in sift_base.fvecs sift_query.fvecs; do
  echo "Extracting ${FILE}..."
  tar -xzf "${ARCHIVE_PATH}" -C "${TMP_DIR}" "sift/${FILE}"
  mv "${TMP_DIR}/sift/${FILE}" "${DATA_DIR}/${FILE}"
done

echo "Done. Files stored in ${DATA_DIR}/"
