"use strict";

// M1〜M7の正規表現と語彙。
//
// crates/suikou-core/src/rules/mod.rs の同名の定数・正規表現を一字一句写した
// ものである。CLAUDE.md が禁じているとおり、ここで語彙を思いつきで増減しては
// ならない。変えるときは Rust 側を先に変え、実測か明文のスタイルガイドの根拠を
// docs/content/*/decisions.md に書いてから、この対応表を追随させる。
//
// 全角数字は ０-９ (０-９) として埋め込んである。ソース中に全角文字を
// 直接書くと、エディタやフォントの都合で読み違えやすいための処置である。

const FULLWIDTH_DIGITS = "０-９";
const KANJI_DIGITS = "一二三四五六七八九十";

/** M6。厳格版に絞る。new / now / future / existing を含めると偽陽性が多い。 */
const TIME_DEPENDENT_JA = [
    "現在",
    "現時点",
    "最新の",
    "新しい",
    "今後",
    "将来的に",
    "まもなく",
    "既存の",
    "目下",
    "現行の",
    // 書いた時点を起点にする語。Google の timeless documentation が
    // recently と soon と new を挙げており、その日本語側にあたる。
    "最近",
    "近年",
    "直近",
    "先日",
    "従来",
    "当面",
    "順次",
];

/**
 * M1。英語の導入文がこれらを含めば完全な導入とみなす。
 * 厳密には定形動詞の有無を見るべきだが、この判定だけで
 * AWS S3 ユーザーガイドと Linux kernel を分離できている。
 */
const EN_LEAD_IN_MARKERS = ["the following", "as follows", "these", "below", "steps", "following"];

const RE_M2_JA = new RegExp(
    `[0-9${FULLWIDTH_DIGITS}${KANJI_DIGITS}]+\\s*[つ個点種類]の\\s*` +
        "(?:理由|方法|ポイント|点|要素|ステップ|手順|方針|特徴|観点|側面|利点|欠点|課題|原則|要因|種類)" +
        `|(?:以下|次)の\\s*[0-9${FULLWIDTH_DIGITS}${KANJI_DIGITS}]+\\s*[つ個点]` +
        `|[0-9${FULLWIDTH_DIGITS}${KANJI_DIGITS}]+\\s*[つ個]あ(?:る|ります)`,
    "g",
);

const RE_M2_EN =
    /\b(two|three|four|five|six|seven|2|3|4|5|6|7)\s+(reasons|ways|steps|things|types|categories|benefits|points|factors|approaches|options|principles|rules|components|parts)\b/gi;

const RE_M3 = new RegExp(
    `^#{1,6}\\s*(?:第?\\s*[0-9${FULLWIDTH_DIGITS}]+\\s*[.．、)）章節]|[0-9${FULLWIDTH_DIGITS}]+\\s*[-–]\\s)`,
);

const RE_M4 = new RegExp(`^\\s*[-*+]\\s+[0-9${FULLWIDTH_DIGITS}]+\\s*[.)．）、]\\s`);

const RE_M6_EN =
    /\b(currently|presently|eventually|soon|latest|newest|newer|as of this writing|at present|in the (?:near )?future|for now|recently|lately|these days|nowadays|so far|for the time being|up to now)\b/gi;

/**
 * 絶対の係留。これがある文では M6 を見送る。
 *
 * 時点に触れること自体は誤りではない。読む時点で意味が変わることが誤りである。
 * 西暦、バージョン番号、番号の付いた文書は、読む時点によらず同じ時点を指す。
 */
const RE_ANCHOR = /(19|20)\d{2}\s*(年|-|\/)|v\d+\.\d+|\d+\.\d+\.\d+|(RFC|KEP|ADR|D-)\s*\d+/;

/** 係留のない相対の期間。数を伴うため語の一覧では網羅できない。 */
const RE_RELATIVE_PERIOD_JA = new RegExp(
    `(直近|過去|ここ|この)\\s*[0-9${FULLWIDTH_DIGITS}${KANJI_DIGITS}]+\\s*(年|ヶ月|か月|箇月|週間|日間)` +
        "|(先|今|来)(週|月|年度|年)",
    "g",
);

const RE_RELATIVE_PERIOD_EN =
    /\b((in|over|for|during)\s+the\s+(past|last)\s+\w+\s+(years?|months?|weeks?|days?)|(last|next)\s+(week|month|quarter|year))\b/gi;

/** M6 の日本語。語の一覧は TIME_DEPENDENT_JA を正本とし、そこから組み立てる。 */
const RE_M6_JA = new RegExp(TIME_DEPENDENT_JA.join("|"), "g");

const RE_M7_JA = /(?:など|等)[。、）)]?\s*$/;
const RE_M7_EN = /(etc\.|and so on|and more)\s*[.)]?\s*$/i;

module.exports = {
    TIME_DEPENDENT_JA,
    EN_LEAD_IN_MARKERS,
    RE_M2_JA,
    RE_M2_EN,
    RE_M3,
    RE_M4,
    RE_M6_EN,
    RE_ANCHOR,
    RE_RELATIVE_PERIOD_JA,
    RE_RELATIVE_PERIOD_EN,
    RE_M6_JA,
    RE_M7_JA,
    RE_M7_EN,
};
