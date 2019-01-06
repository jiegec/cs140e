#! /bin/bash

function cleanup_and_exit() {
  kill $!
  exit $1
}

# generates a random base64 encoded string between 1 and 512 bytes
function rand_string() {
  base64 < /dev/urandom | head -c $((1 + RANDOM % 512))
}

# Use color when outputting to the terminal.
if [ -t 1 ]; then
  KNRM="\x1B[0m"; KRED="\x1B[31m"; KGRN="\x1B[32m"; KBLU="\x1B[34m"
else
  KNRM=""; KRED=""; KGRN=""; KBLU=""
fi

if ! command -v socat > /dev/null 2>&1; then
  echo >&2 "error: the 'socat' command is required but not installed"
  echo >&2 "help: install the 'socat' package using your package manager"
  exit 1
fi

echo -e "${KBLU}Compiling project with 'cargo build'...${KNRM}"
if ! cargo build; then
  echo -e "${KRED}ERROR: ttywrite compilation failed${KNRM}" >&2
fi

echo -e "${KBLU}Opening PTYs...${KNRM}"
PARAMS="pty,echo=0,raw,ispeed=115200,ospeed=115200,parenb=0,cs8,cstopb=0"
socat -u ${PARAMS},link=input ${PARAMS},link=output &
sleep 1

if [[ "$(uname)" = "Darwin" ]]; then
  # having GNU coreutils in my PATH
  stty -F input min 0 time 1
  stty -F output min 0 time 1
else
  stty -F input min 0 time 1
  stty -F output min 0 time 1
fi

for i in {1..10}; do
  echo -e "${KBLU}Running test ${i}/10.${KNRM}"

  input=$(rand_string)
  echo -n "${input}" | ./target/debug/ttywrite -r input
  output=$(cat output)
  if [[ "${output}" != "${input}" ]]; then
    echo -e "${KRED}ERROR: input and output differ${KNRM}" >&2
    echo "${input} != ${output}" >&2
    cleanup_and_exit 1
  fi
done

echo -e "${KGRN}SUCCESS${KNRM}"
cleanup_and_exit 0
