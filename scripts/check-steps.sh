#!/usr/bin/env bash
# Vérifie le dispositif de travail étape par étape : engendre un dossier de travail
# temporaire (le travail/ du dépôt n'est pas touché), puis pour chaque étape
# 0..LAST_STEP (croissant), goto 0 puis next doivent compiler et laisser cargo test rouge
# sur l'étape n seulement (les todo!() atteints sont tous ceux de l'étape n, jamais d'une
# étape antérieure) ; solve <n> doit ensuite ramener cargo test au vert. Au-delà de la
# dernière étape, goto et next doivent refuser sans rien changer.
# À la dernière étape : aucun avertissement, l'analyse complète tourne sur un jeu de
# données généré, et reset/solve rouvrent puis referment bien un trou.
#
# Usage : scripts/check-steps.sh   (depuis n'importe où dans le dépôt)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

red=$'\033[31m'
green=$'\033[32m'
reset=$'\033[0m'

fail() {
    echo "${red}!! $*${reset}" >&2
    exit 1
}
ok() {
    echo "   ${green}ok${reset} $*"
}

# La dernière étape, déduite du manifeste : sa feature `stepN` la plus haute est la
# sentinelle, LAST_STEP + 1 (le test the_manifest_ends_with_the_sentinel de xtask le
# garantit ; goto et next le confirment ci-dessous).
sentinel="$(sed -n 's/^step\([0-9][0-9]*\) = .*/\1/p' sentinelle/Cargo.toml | sort -n | tail -1)"
[ -n "$sentinel" ] || fail "aucune feature stepN dans sentinelle/Cargo.toml"
last_step=$((sentinel - 1))

work="$(mktemp -d)"
cleanup() {
    cd "$root"
    rm -rf "$work"
}
trap cleanup EXIT

# Étapes des todo!() atteints par les tests rouges, une par ligne, triées.
reached_steps() {
    echo "$1" | grep -oE 'étape [0-9]+ — voir le commentaire' | grep -oE '[0-9]+' | sort -un
}

echo "==> étapes 0 à ${last_step}"
echo "==> cargo xtask start --dir ${work}/travail"
cargo xtask start --dir "$work/travail" >/dev/null
cd "$work/travail"
[ ! -e solutions ] || fail "travail/ contient solutions/ : il ne doit jamais être copié"
cmp -s rust-toolchain.toml "$root/rust-toolchain.toml" \
    || fail "travail/ n'a pas la même toolchain épinglée (rust-toolchain.toml) que le dépôt"
ok "travail/ engendré, sans solutions/, même toolchain ($(rustc --version | cut -d' ' -f2))"

echo "==> goto refuse l'étape ${sentinel}, qui n'existe pas encore"
if cargo xtask goto "$sentinel" >/dev/null 2>&1; then
    fail "cargo xtask goto ${sentinel} réussit : LAST_STEP de xtask et sentinelle/Cargo.toml divergent"
fi
grep -qx 'default = \["step0"\]' sentinelle/Cargo.toml \
    || fail "goto ${sentinel} a modifié l'étape courante alors qu'il a échoué"
ok "goto ${sentinel} refusé, toujours à l'étape 0"

for n in $(seq 0 "$last_step"); do
    echo "==> étape ${n}"

    # l'étape 0 par son numéro, les suivantes avec `next` (après solve de la précédente)
    if [ "$n" -eq 0 ]; then
        echo "    goto ${n}"
        cargo xtask goto "$n" >/dev/null
    else
        echo "    next"
        cargo xtask next >/dev/null
    fi
    grep -qx "default = \[\"step${n}\"\]" sentinelle/Cargo.toml \
        || fail "étape ${n} : sentinelle/Cargo.toml n'est pas passé à step${n}"

    echo "    build"
    cargo build --all-targets >/dev/null 2>&1 || fail "étape ${n} : cargo build échoue avant même solve"

    echo "    test (rouge attendu, sur l'étape ${n} seulement)"
    set +e
    out="$(cargo test 2>&1)"
    status=$?
    set -e
    [ "$status" -ne 0 ] || fail "étape ${n} : cargo test déjà vert alors que l'étape vient d'être ouverte"
    steps="$(reached_steps "$out")"
    [ -n "$steps" ] || { echo "$out" | tail -20 >&2; fail "étape ${n} : cargo test rouge sans todo!() reconnaissable"; }
    [ "$steps" = "$n" ] || fail "étape ${n} : todo!() atteints sur les étapes $(echo $steps), au lieu de ${n} seule"
    ok "rouge sur l'étape ${n}"

    echo "    solve ${n}"
    cargo xtask solve "$n" >/dev/null

    echo "    test (doit être entièrement vert)"
    cargo test >/dev/null 2>&1 || fail "étape ${n} : cargo test rouge après solve"
    ok "cargo test vert"
done

echo "==> dernière étape : aucun avertissement"
warnings="$(cargo build --all-targets 2>&1 | grep -c '^warning' || true)"
[ "$warnings" -eq 0 ] || fail "${warnings} avertissement(s) une fois toutes les étapes résolues"
ok "aucun avertissement"

echo "==> l'analyse complète tourne sur un jeu généré"
cargo run --quiet --release -p sentinelle-gen -- --out data >/dev/null
cargo run --quiet --release -p sentinelle -- data >/dev/null 2>&1 || fail "cargo run -p sentinelle -- data échoue"
ok "cargo run -p sentinelle -- data"

echo "==> next refuse d'aller au-delà de l'étape ${last_step}"
if cargo xtask next >/dev/null 2>&1; then
    fail "cargo xtask next réussit à la dernière étape"
fi
grep -qx "default = \[\"step${last_step}\"\]" sentinelle/Cargo.toml \
    || fail "next a modifié l'étape courante alors qu'il a échoué"
ok "next refusé, toujours à l'étape ${last_step}"

echo "==> reset ${last_step} rouvre les trous, solve ${last_step} les referme"
cargo xtask reset "$last_step" >/dev/null
if cargo test >/dev/null 2>&1; then
    fail "cargo test vert alors que l'étape ${last_step} vient d'être rouverte"
fi
ok "rouge une fois l'étape rouverte"
cargo xtask solve "$last_step" >/dev/null
cargo test >/dev/null 2>&1 || fail "cargo test rouge après solve ${last_step}"
ok "vert de nouveau"

echo
echo "${green}==> étapes 0 à ${last_step} toutes vérifiées${reset}"
