# AIVoiceVox - A.I.Voice to Voicevox bridge

AIVoiceVox は[A.I.Voice](https://aivoice.jp/)を[Voicevox](https://voicevox.hiroshiba.jp/)にマルチエンジンとして読み込ませるブリッジです。

> **Warning**
> このエンジンは非公式です。
> また、このプラグインの HTTP サーバーを複数人に公開することは[利用規約](https://aivoice.jp/manual/editor/api.html#termsandconditions)に違反する可能性があります。

## インストール

[Releases](https://github.com/sevenc-nanashi/aivoice-vox/releases)から最新のバージョンの vvpp ファイルをダウンロードし、「エンジンの管理」/「追加」/「VVPP ファイル」からインストールしてください。

## 注意

- このプラグインの起動中は A.I.Voice の設定を書き換えます。A.I.Voice を終了すると元に戻ります。
  もし異常終了した場合は、もう一度プラグインを起動してください。可能な限り元の設定に戻します。

- A.I.Voice 内に作成される「AIVoiceVox」ボイスプリセットは削除しないでください。削除すると次の起動時まで AIVoiceVox が正常に動作しません。

- 開発者が感情を持つキャラクターを持っていないため、感情のテストはしていません。

## ライセンス

MIT License で公開しています。詳しくは[LICENSE](LICENSE)をご覧ください。
生成された音声については、A.I.Voice の利用規約に従ってください。
このブリッジ自体にはクレジット表記は必要ありませんが、このリポジトリのリンクを貼ったりや紹介動画（TODO）をおや作品登録していただくと助かります。
