# AIVoiceVox - A.I.Voice to Voicevox bridge

AIVoiceVox は[A.I.Voice](https://aivoice.jp/)を[Voicevox](https://voicevox.hiroshiba.jp/)にマルチエンジンとして読み込ませるブリッジです。

> [!WARNING]
> このエンジンは非公式です。
> また、このプラグインの HTTP サーバーを複数人に公開することは[利用規約](https://aivoice.jp/manual/editor/api.html#termsandconditions)に違反する可能性があります。

## TODO

- [ ] ボイスフュージョン（モーフィングに落とし込む？）
- [ ] 感情パラメータ（モーフィング？）
- [ ] CI（cppを置き換える？）

## インストール

1. [Releases](https://github.com/sevenc-nanashi/aivoice-vox/releases)から最新のバージョンの vvpp ファイルをダウンロードしてください。
2. Voicevoxの設定を開き、「実験的機能」から「マルチエンジン機能」を有効化してください。
3. 「エンジンの管理」/「追加」/「VVPP ファイル」からインストールしてください。

## 注意

- このプラグインの起動中は A.I.Voice の設定を書き換えます。A.I.Voice を終了すると元に戻ります。
  もし異常終了した場合は、もう一度プラグインを起動してください。可能な限り元の設定に戻します。

- A.I.Voice 内に作成される「AIVoiceVox」ボイスプリセットは削除しないでください。削除すると次の起動時まで AIVoiceVox が正常に動作しません。

- 開発者が感情を持つキャラクターを持っていないため、感情のテストはしていません。

## ライセンス

MIT License で公開しています。詳しくは[LICENSE](LICENSE)をご覧ください。  
生成された音声については、A.I.Voice の利用規約に従ってください。  
このブリッジ自体にはクレジット表記は必要ありませんが、このリポジトリのリンクを貼ったり[紹介動画](https://www.nicovideo.jp/watch/sm43073706?ref=nicoiphone_other)を親作品登録していただいたりすると助かります。
