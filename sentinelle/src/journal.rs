//! Un fichier de journal entier : les entrées analysées et les lignes rejetées.
//!
//! Le fichier est lu **une fois** en mémoire (`Vec<u8>`) ; les entrées en empruntent
//! ensuite des tranches. Le texte doit donc vivre au moins aussi longtemps que le
//! [`Journal`] qui le décrit : c'est ce que dit le `'a` de [`parse_lines`].

use crate::ParseError;
use crate::auth::{AuthEntry, parse_auth_line};
use crate::web::{WebEntry, parse_web_line};
use std::path::Path;

/// Le résultat de l'analyse d'un fichier.
#[derive(Debug)]
pub struct Journal<T> {
    pub entries: Vec<T>,
    /// Lignes rejetées : numéro (à partir de 1) et raison.
    pub errors: Vec<(usize, ParseError)>,
}

/// Lit un fichier entier, sans exiger qu'il soit de l'UTF-8 valide.
pub fn load(path: &Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// Les lignes d'un texte brut, sans leur `\n` final.
///
/// Contrairement à `str::lines`, travaille sur des octets : une ligne d'UTF-8 invalide
/// n'empêche pas de lire les suivantes.
pub fn lines(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    text.split_inclusive(|&b| b == b'\n')
        .map(|line| line.strip_suffix(b"\n").unwrap_or(line))
}

/// Analyse chaque ligne de `text` avec `parse`.
///
/// Les entrées obtenues empruntent `text` (durée de vie `'a`). Une ligne qui n'est pas de
/// l'UTF-8 valide est rejetée avec [`ParseError::InvalidUtf8`], sans interrompre la
/// lecture.
#[cfg_attr(not(feature = "step3"), allow(unused_variables))] // trou étape 3
pub fn parse_lines<'a, T>(
    text: &'a [u8],
    parse: impl Fn(&'a str) -> Result<T, ParseError>,
) -> Journal<T> {
    // TODO-STEP:3 Parcourir `lines(text)` en numérotant les lignes, convertir chacune en
    // `&str` avec `std::str::from_utf8`, l'analyser, et ranger le résultat dans `entries`
    // ou dans `errors`.
    // SOLUTION-BEGIN
    let mut journal = Journal {
        entries: Vec::new(),
        errors: Vec::new(),
    };
    for (index, line) in lines(text).enumerate() {
        let parsed = std::str::from_utf8(line)
            .map_err(|_| ParseError::InvalidUtf8)
            .and_then(&parse);
        match parsed {
            Ok(entry) => journal.entries.push(entry),
            Err(error) => journal.errors.push((index + 1, error)),
        }
    }
    journal
    // SOLUTION-END
}

/// Analyse le contenu de `auth.log`.
pub fn parse_auth(text: &[u8]) -> Journal<AuthEntry<'_>> {
    parse_lines(text, parse_auth_line)
}

/// Analyse le contenu de `access.log`.
pub fn parse_web(text: &[u8]) -> Journal<WebEntry<'_>> {
    parse_lines(text, parse_web_line)
}
