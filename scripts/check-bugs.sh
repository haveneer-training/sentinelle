#!/usr/bin/env bash
# Vérifie le TP du module 6 dans les deux sens :
#   1. sentinelle-turbo (bugs/) présente bien chaque symptôme de formateur/BUGS.md, sur
#      les jeux de docs/modules/module-6.md (une modification de bugs/ ou du générateur
#      ne doit pas faire disparaître un ticket par mégarde) ;
#   2. le corrigé formateur/m6-turbo-fixed passe fmt, clippy et ses tests.
#
# Le ticket 4 (interblocage) dépend de l'ordonnancement : s'il ne se reproduit pas en
# quelques essais, le script le signale sans échouer.
#
# Usage : scripts/check-bugs.sh   (depuis n'importe où dans le dépôt)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

red=$'\033[31m'
green=$'\033[32m'
yellow=$'\033[33m'
reset=$'\033[0m'

fail() {
    echo "${red}!! $*${reset}" >&2
    exit 1
}
ok() {
    echo "   ${green}ok${reset} $*"
}
warn() {
    echo "   ${yellow}??${reset} $*"
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# run_for SECONDES OUT ERR CMD... : lance CMD, le tue au bout de SECONDES.
# Code de retour : celui de CMD, ou 124 s'il a été tué (comme timeout(1), absent de macOS).
run_for() {
    local seconds="$1" out="$2" err="$3"
    shift 3
    "$@" >"$out" 2>"$err" &
    local pid=$! waited=0
    while kill -0 "$pid" 2>/dev/null; do
        if [ "$waited" -ge $((seconds * 10)) ]; then
            kill -9 "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
            return 124
        fi
        sleep 0.1
        waited=$((waited + 1))
    done
    wait "$pid"
}

# expect_panic NOM MOTIF BINAIRE ARGS... : le binaire doit paniquer avec MOTIF sur stderr.
expect_panic() {
    local name="$1" pattern="$2"
    shift 2
    local status=0
    run_for 60 "$work/out" "$work/err" "$@" || status=$?
    [ "$status" -ne 0 ] || fail "${name} : $* réussit au lieu de paniquer"
    [ "$status" -ne 124 ] || fail "${name} : $* bloqué au lieu de paniquer"
    grep -q "$pattern" "$work/err" || { cat "$work/err" >&2; fail "${name} : panique sans « ${pattern} »"; }
    ok "${name} : $(grep -o "$pattern" "$work/err" | head -1)"
}

echo "==> jeux de données (docs/modules/module-6.md)"
cargo build --quiet --release -p sentinelle-gen
gen="$root/target/release/sentinelle-gen"
"$gen" --out "$work/data" --reveal >/dev/null
"$gen" --out "$work/data-propre" --malformed 0 >/dev/null
"$gen" --out "$work/data-abime" --invalid-utf8 --malformed 0.01 >/dev/null
brute_ip="$(sed -n 's/^| Force brute |.*| `\([0-9.]*\)` →.*/\1/p' "$work/data/GROUND_TRUTH.md")"
[ -n "$brute_ip" ] || fail "adresse de la force brute introuvable dans GROUND_TRUTH.md"
ok "data, data-propre, data-abime (force brute : ${brute_ip})"

echo "==> sentinelle-turbo (bugs/) : les symptômes"
cargo build --quiet -p sentinelle-turbo
cargo build --quiet --release -p sentinelle-turbo
debug="$root/target/debug/sentinelle-turbo"
release="$root/target/release/sentinelle-turbo"

expect_panic "ticket 1 (debug)" "attempt to add with overflow" "$debug" report "$work/data-propre"
run_for 60 "$work/out" "$work/err" "$release" report "$work/data-propre" || fail "ticket 1 (release) : échec"
grep -qx "0 alerte(s)" "$work/out" || { cat "$work/out" >&2; fail "ticket 1 (release) : des alertes, le volume ne reboucle plus"; }
ok "ticket 1 (release) : 0 alerte(s)"

expect_panic "ticket 2" "byte index [0-9]* is out of bounds" "$debug" report "$work/data"
expect_panic "ticket 3" "stream did not contain valid UTF-8" "$debug" report "$work/data-abime"

reproduced=""
for attempt in 1 2 3 4 5; do
    status=0
    run_for 15 "$work/out" "$work/err" "$release" stats "$work/data" --threads 8 || status=$?
    if [ "$status" -eq 124 ]; then
        reproduced="$attempt"
        break
    fi
    [ "$status" -eq 0 ] || { cat "$work/err" >&2; fail "ticket 4 : stats échoue au lieu de bloquer"; }
done
if [ -n "$reproduced" ]; then
    ok "ticket 4 : stats --threads 8 bloqué (essai ${reproduced})"
else
    warn "ticket 4 : interblocage non reproduit en 5 essais sur cette machine ($(getconf _NPROCESSORS_ONLN 2>/dev/null || echo '?') cœurs)"
fi
run_for 60 "$work/out" "$work/err" "$release" stats "$work/data" --threads 1 || fail "ticket 4 : stats --threads 1 échoue ou bloque"
ok "ticket 4 : stats --threads 1 rend la main"

status=0
run_for 15 "$work/out" "$work/err" "$release" stream "$work/data" || status=$?
[ "$status" -eq 124 ] || fail "ticket 5 : stream rend la main (code ${status}) au lieu de se figer"
grep -q "blocs agrégés" "$work/err" || fail "ticket 5 : stream figé avant d'agréger le moindre bloc"
ok "ticket 5 : stream figé après $(tr '\r' '\n' <"$work/err" | grep -o '[0-9]* blocs' | tail -1)"

run_for 60 "$work/out" "$work/err" "$release" stats "$work/data" --threads 1
! grep -q "^ *${brute_ip//./\\.} " "$work/out" || fail "ticket bonus : ${brute_ip} présente dans le top de stats"
ok "ticket bonus : ${brute_ip} absente du top de stats"

echo "==> corrigé (formateur/m6-turbo-fixed)"
manifest=formateur/m6-turbo-fixed/Cargo.toml
cargo fmt --manifest-path "$manifest" --check || fail "corrigé mal formaté"
cargo clippy --quiet --manifest-path "$manifest" --all-targets -- -D warnings || fail "clippy sur le corrigé"
cargo test --quiet --manifest-path "$manifest" || fail "tests du corrigé"
ok "fmt, clippy, tests"

echo
echo "${green}==> module 6 vérifié${reset}"
