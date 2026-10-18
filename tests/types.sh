#!/bin/sh
# Check the Luau definitions: the sample script passes `--!strict`, the two mistakes are reported.
set -eu
cd "$(dirname "$0")/.."
args="--platform=standard --flag:LuauSolverV2=true --definitions=decimal.d.luau"
luau-lsp analyze $args tests/types/ok.luau
for bad in tests/types/bad_eq.luau tests/types/bad_lt.luau; do
    if luau-lsp analyze $args "$bad" >/dev/null 2>&1; then
        echo "expected a type error in $bad" >&2
        exit 1
    fi
done
echo "luau types ok"
