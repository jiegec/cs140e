#! /bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

function usage() {
  echo "Usage: $0 [-h, -v] [filter]"
}

VERBOSE=false
while getopts "vh" opt; do
  case $opt in
    v) VERBOSE=true
      ;;
    h) usage ; exit 0 ;;
    [?]) usage >&2 ; exit 1 ;;
    :)
      echo "Option -$OPTARG requires an argument." >&2
      exit 1
      ;;
  esac
done

FILTER=${@:$OPTIND:1}

function verbose() {
  if $VERBOSE -eq 0; then
    echo "${@}"
  fi
}

# Use color when outputting to the terminal.
if [ -t 1 ]; then
  KNRM="\x1B[0m"
  KRED="\x1B[31m"
  KBRED="\x1B[91m"
  KGRN="\x1B[32m"
  KBLU="\x1B[34m"
  KWHT="\x1B[37m"
  RUSTC_FLAGS="--color=always"
else
  KNRM=""
  KRED=""
  KBRED=""
  KGRN=""
  KBLU=""
  KWHT=""
  RUSTC_FLAGS=""
fi

function compile_test() {
  local file="${1}"
  local should_compile=$2
  local should_run_pass=$3
  local file_dir="${file%/*}"
  local filename="${file_dir##*/}/${file##*/}"

  if [ -n "${FILTER}" ] && ! [[ "${filename}" == *"${FILTER}"* ]]; then
    verbose -e "${KBLU}SKIPPING: ${KWHT}${filename}${KNRM}"
    return 0
  fi

  stderr=$(rustc "${file}" $RUSTC_FLAGS -Z no-trans 2>&1)
  result=$?

  if $should_compile && [ $result -ne 0 ]; then
    echo -e "${KRED}ERROR: ${KBRED}${filename}${KRED} failed to compile!${KNRM}"
    verbose -e "${KWHT}---------------------- stderr --------------------------${KNRM}"
    verbose -e "${stderr}"
  fi

  if ! $should_compile; then
    if [ $result -eq 0 ]; then
      echo -e "${KRED}ERROR: ${KBRED}${filename}${KRED} compiled unexpectedly!${KNRM}"
      result=1
    else
      result=0
    fi
  fi

  if [ $result -eq 0 ] && $should_run_pass; then
    local executable="${file}.out"
    rustc "${file}" $RUSTC_FLAGS -o "${executable}" 2>/dev/null 1>/dev/null
    stdout=$("${executable}" 2>&1)
    result=$?

    rm -f "${executable}"
    if [ $result -ne 0 ]; then
      echo -e "${KRED}ERROR: ${KBRED}${filename}${KRED} failed execution!${KNRM}"
      verbose -e "${KWHT}---------------------- output --------------------------${KNRM}"
      verbose -e "${stdout}"
    fi
  fi

  if [ $result -eq 0 ]; then
    let PASSES+=1
    echo -e "${KGRN}SUCCESS: ${KWHT}${filename}${KNRM}"
  else
    let FAILURES+=1
  fi

  echo ""
  return $result
}

let PASSES=0
let FAILURES=0

for file in "${SCRIPT_DIR}/compile-pass/"*.rs; do
  compile_test "${file}" true false
done

for file in "${SCRIPT_DIR}/compile-fail/"*.rs; do
  compile_test "${file}" false false
done

for file in "${SCRIPT_DIR}/run-pass/"*.rs; do
  compile_test "${file}" true true
done

echo -e "${KGRN}${PASSES} passes${KNRM}, ${KRED}${FAILURES} failures${KNRM}"
