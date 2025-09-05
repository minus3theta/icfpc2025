## mock-server

ローカルで動く問題サーバ

`/select` `/explore` `/guess` に加えて、guess しても地図がリセットされない `/guess-keep` があります。

select と guess では地図を info で出力します。

### 動かしかた
```shell
cargo run --bin mock-server
```

### リクエストの投げ方
```shell
curl -X POST -H "Content-Type: application/json" http://localhost:8080/select -d '{"id": "hoge", "problemName": "primus"}'
```
