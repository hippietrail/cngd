use harper_core::DictWordMetadata;

pub type PosPredicate = fn(&DictWordMetadata) -> bool;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pos {
    Adjective,
    Adverb,
    Conjunction,
    Determiner,
    Noun,
    Preposition,
    Pronoun,
    ProperNoun,
    Verb,
}

pub struct POSInfo {
    pub _name: &'static str,
    pub _ptb: &'static str, // Penn Treebank
    pub letter: &'static str,
    pub _emoji: &'static str,
}

pub const POS_DEFINITIONS: &[(Pos, PosPredicate)] = &[
    (Pos::Noun, |m| m.is_noun() && !m.is_proper_noun()),
    (Pos::ProperNoun, DictWordMetadata::is_proper_noun),
    (Pos::Verb, DictWordMetadata::is_verb),
    (Pos::Adjective, DictWordMetadata::is_adjective),
    (Pos::Adverb, DictWordMetadata::is_adverb),
    (Pos::Conjunction, DictWordMetadata::is_conjunction),
    (Pos::Determiner, DictWordMetadata::is_determiner),
    (Pos::Preposition, |m| m.preposition),
    (Pos::Pronoun, DictWordMetadata::is_pronoun),
];

pub fn pos_info(pos: &Pos) -> POSInfo {
    match pos {
        Pos::Noun => POSInfo {
            letter: "N",
            _ptb: "NN",
            _emoji: "📦",
            _name: "noun",
        },
        Pos::ProperNoun => POSInfo {
            letter: "O",
            _ptb: "NNP",
            _emoji: "📛",
            _name: "proper noun",
        },
        Pos::Verb => POSInfo {
            letter: "V",
            _ptb: "VB",
            _emoji: "🏃",
            _name: "verb",
        },
        Pos::Adjective => POSInfo {
            letter: "J",
            _ptb: "JJ",
            _emoji: "🌈",
            _name: "adjective",
        },
        Pos::Adverb => POSInfo {
            letter: "R",
            _ptb: "RB",
            _emoji: "🤷",
            _name: "adverb",
        },
        Pos::Conjunction => POSInfo {
            letter: "C",
            _ptb: "CC",
            _emoji: "🔗",
            _name: "conjunction",
        },
        Pos::Determiner => POSInfo {
            letter: "D",
            _ptb: "DT",
            _emoji: "👉",
            _name: "determiner",
        },
        Pos::Preposition => POSInfo {
            letter: "P",
            _ptb: "IN",
            _emoji: "📥",
            _name: "preposition",
        },
        Pos::Pronoun => POSInfo {
            letter: "I",
            _ptb: "PRP",
            _emoji: "👤",
            _name: "pronoun",
        },
    }
}
