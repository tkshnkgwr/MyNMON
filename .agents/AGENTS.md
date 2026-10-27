# MyNMON 開発ガイドライン (AGENTS.md)

本プロジェクトにおける大賢者の開発指示書です。
最優先ルール（コミット禁止・テーブル整列等）は [RULES.md](../RULES.md) および [GEMINI.md](../GEMINI.md) を参照してください。

## 🎯 開発・品質ルール

- **技術解説方針（他言語経験への配慮）**:
  - ボスの他言語経験を尊重し、Rust固有の概念は一般的なプログラミング概念に置き換えて分かりやすく解説すること。
- **事前検証とスキル委譲**:
  - コード変更時は [.agents/skills/rust-verification/SKILL.md](file:///.agents/skills/rust-verification/SKILL.md) を実行すること。
  - Markdown（`*.md`）のみの編集時は事前検証を省略すること。
- **コード規模とリファクタリング**:
  - 単一ソースが1000行を超えた場合はモジュール分割リファクタリングを積極的に提案すること。
- **コミット手順（スキル委譲）**:
  - ボスから指示があった場合のみ、[.agents/skills/git-commit/SKILL.md](file:///.agents/skills/git-commit/SKILL.md) に従って実行すること。
