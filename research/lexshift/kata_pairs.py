"""漢語とカタカナ語の候補を、分野をまたいだ実測で採否する。

候補の列挙と採否の判断を分ける。候補は人が挙げてよいが、採るかどうかは測った値で決める。
分野ごとに逆転する語はあるが、合計で優勢なら従う。

使い方: python kata_pairs.py <プロ日本語のディレクトリ> <出力 toml>
"""
import collections, datetime, pathlib, re, sys
import fugashi
from chunks import aligned, kind

MIN_TOTAL = 20     # 対の合計の出現数の下限。少ない語で決めない
MIN_SHARE = 2.0    # カタカナ側が漢語側の何倍以上であれば優勢とみなすか
MAX_KANGO = 60     # 漢語側の出現数の上限。約29万語に対する数である
#
# 上限を置く理由は多義語にある。「対象」「対応」「記録」はプロもよく使うが、
# それはカタカナ語と同じ意味で使っているとは限らない。
# プロがその漢語をほとんど使っていない場合に限れば、置き換えの対応として信頼できる。
TAGGER = fugashi.Tagger()
INLINE = re.compile(r"`[^`]*`|\{\{[^}]*\}\}|\[([^\]]*)\]\([^)]*\)|<[^>]+>")

# 候補。技術文書で両方の書き方が見られる組を挙げる。採否はこの下の測定が決める。
CANDIDATES = [
    ("版", "バージョン"), ("利用者", "ユーザー"), ("使用者", "ユーザー"),
    ("性能", "パフォーマンス"), ("誤り", "エラー"), ("標本", "サンプル"),
    ("一覧", "リスト"), ("目録", "リスト"), ("既定", "デフォルト"),
    ("仕組み", "メカニズム"), ("対応", "サポート"), ("支援", "サポート"),
    ("資源", "リソース"), ("節点", "ノード"), ("容器", "コンテナ"),
    ("集団", "クラスター"), ("役割", "ロール"), ("記録", "ログ"),
    ("接続口", "ポート"), ("経路", "パス"), ("経路指定", "ルーティング"),
    ("配備", "デプロイ"), ("展開", "デプロイ"), ("移行", "マイグレーション"),
    ("枠組み", "フレームワーク"), ("書式", "フォーマット"), ("形式", "フォーマット"),
    ("階層", "レイヤー"), ("層", "レイヤー"), ("段階", "ステップ"),
    ("手順", "ステップ"), ("試験", "テスト"), ("検査", "テスト"),
    ("要求", "リクエスト"), ("応答", "レスポンス"), ("問い合わせ", "クエリ"),
    ("引数", "パラメーター"), ("媒介変数", "パラメーター"), ("設定", "コンフィグ"),
    ("走査", "スキャン"), ("索引", "インデックス"), ("見出し語", "キー"),
    ("鍵", "キー"), ("値打ち", "バリュー"), ("状態", "ステータス"),
    ("規模", "スケール"), ("拡張", "スケール"), ("負荷", "ロード"),
    ("均衡", "バランス"), ("監視", "モニタリング"), ("観測", "モニタリング"),
    ("保守", "メンテナンス"), ("更新", "アップデート"), ("改版", "アップグレード"),
    ("復元", "リストア"), ("控え", "バックアップ"), ("予備", "バックアップ"),
    ("待ち行列", "キュー"), ("緩衝", "バッファ"), ("記憶域", "ストレージ"),
    ("保存域", "ストレージ"), ("網", "ネットワーク"), ("通信", "ネットワーク"),
    ("交渉", "ネゴシエーション"), ("認証", "オーセンティケーション"),
    ("権限", "パーミッション"), ("許可", "パーミッション"),
    ("模型", "モデル"), ("模擬", "シミュレーション"), ("併合", "マージ"),
    ("分岐", "ブランチ"), ("差分", "ディフ"), ("補綴", "パッチ"),
    ("修正版", "パッチ"), ("道具", "ツール"), ("工具", "ツール"),
    ("環境", "エンバイロメント"), ("事例", "ケース"), ("場合", "ケース"),
    ("実例", "インスタンス"), ("実体", "インスタンス"), ("対象", "オブジェクト"),
    ("属性", "アトリビュート"), ("注釈", "アノテーション"), ("札", "ラベル"),
    ("目印", "マーカー"), ("応用", "アプリケーション"), ("算法", "アルゴリズム"),
    ("符号", "コード"), ("符号化", "エンコード"), ("復号", "デコード"),
    ("圧縮", "コンプレッション"), ("速さ", "スピード"), ("遅延", "レイテンシ"),
    ("帯域", "バンド幅"), ("段落", "パラグラフ"), ("比率", "レート"),
]


def norm(lemma):
    return lemma.split("-")[0]


def counts(paths):
    c = collections.Counter()
    for p in paths:
        for ch in aligned(pathlib.Path(p).read_text(errors="ignore")):
            if kind(ch) == "code":
                continue
            for w in TAGGER(INLINE.sub(lambda m: m.group(1) or " ", ch)):
                f = w.feature
                if f.pos1 in ("名詞", "動詞", "形容詞", "形状詞", "副詞"):
                    c[norm(f.lemma or w.surface)] += 1
    return c


def main():
    d, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    groups = collections.defaultdict(list)
    for p in sorted(d.glob("*.md")):
        g = p.name.split("__")[0]
        if g in ("k8s", "mdn", "vue"):
            groups[g].append(p)
    per = {g: counts(ps) for g, ps in groups.items()}
    total = collections.Counter()
    for c in per.values():
        total.update(c)

    kept, dropped = [], []
    for kan, kata in CANDIDATES:
        a, b = total[kan], total[kata]
        detail = " ".join(f"{g}={per[g][kan]}/{per[g][kata]}" for g in sorted(per))
        share = b / a if a else float("inf")
        if a + b < MIN_TOTAL or a > MAX_KANGO or share < MIN_SHARE:
            dropped.append((kan, kata, a, b, detail))
            continue
        kept.append((kan, kata, a, b, detail))
    lines = [
        "# 漢語よりカタカナ語が優勢な対応表。",
        "# 候補は人が挙げ、採否は分野をまたいだ実測が決めた。",
        "# 分野ごとに逆転する語はあるが、合計で優勢なら従う。",
        "# 作り直す手順は research/lexshift/kata_pairs.py にある。",
        "",
        "[meta]",
        f'measured = "{datetime.date.today()}"',
        f"min_total = {MIN_TOTAL}",
        f"min_share = {MIN_SHARE}",
        f"max_kango = {MAX_KANGO}",
        'sources = ["kubernetes/website(ja)", "mdn/translated-content(ja)", "vuejs-translations/docs-ja"]',
        "",
        "[pairs]",
    ]
    for kan, kata, a, b, detail in sorted(kept, key=lambda r: -(r[3] + r[2])):
        lines.append(f'"{kan}" = "{kata}"  # 漢語 {a} / カタカナ {b}  {detail}')
    out.write_text("\n".join(lines) + "\n")
    print(f"採った対: {len(kept)} / 落とした対: {len(dropped)}")
    print("\n採用:")
    for kan, kata, a, b, detail in sorted(kept, key=lambda r: -(r[3] + r[2]))[:24]:
        print(f"  {kan:<8}→ {kata:<14} 漢語 {a:5d} / カタカナ {b:5d}   {detail}")
    print("\n棄却（漢語も使われている、合計で漢語が優勢、または標本が足りない）:")
    for kan, kata, a, b, detail in sorted(dropped, key=lambda r: -(r[2] + r[3]))[:12]:
        print(f"  {kan:<8}→ {kata:<14} 漢語 {a:5d} / カタカナ {b:5d}   {detail}")


if __name__ == "__main__":
    main()
