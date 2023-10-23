# Ju. N. Owen

東方獣王園の非公式オンライン対戦ツールです。

非公式のツールです。**自己責任で使用してください。**

公式のオンライン対戦のマッチングや同期機構とは異なる、独自の仕組みでオンライン対戦を実現します。
adonis や th075caster と同じような仕組みで動作します。


## 特徴

- 公式のオンライン対戦よりもずれにくい
- ゲーム中にディレイを変更できる
- サーバーなしで接続できる
- 観戦ができる（予定


## 使用方法

現在マッチングサーバーは未実装なので、チャットなどで対戦相手と接続情報を交換する必要があります。

1. th19.exe のフォルダーに、d3d9.dll と modules フォルダーを置きます。
2. 獣王園を起動します。
3. 上手くいくと獣王園のタイトル画面の項目に「Ju.N.Owen」が追加されるので、それを選択します。
4. ホストとして接続を待ち受ける場合は「Connect as Host」を、
   ゲストとして接続する場合は「Connect as Guset」を選択します。
    - ホスト
        1. `<offer>********</offer>` という長い文字列が表示され、自動的にクリップボードにコピーされるので、
           この文字列を Discord 等を使って対戦相手に送信してください。
           「Copy your code」を選択すると再度クリップボードにコピーされます。
        2. 対戦相手から `<answer>********</answer>` という文字列を受け取り、
           クリップボードにコピーしてください。
        3. 「Paste guest's code」を選択してください。
        4. うまくいけば難易度選択に遷移し、対戦が開始されます。
    - ゲスト
        1. 対戦相手から `<offer>********</offer>` という文字列を受け取り、クリップボードにコピーしてください。
        2. 決定ボタンを押すと、クリップボードの内容が入力されます。
        3. `<answer>********</answer>` という長い文字列が表示され、自動的にクリップボードにコピーされるので、
           この文字列を Discord 等を使って対戦相手に送信してください。
           決定ボタンを押すと再度クリップボードにコピーされます。
        4. うまくいけば難易度選択に遷移し、対戦が開始されます。
4 ホストはゲーム中に数字キーの0-9でディレイを変更できます。


### 補足

- ポート開放は必要ありません。開放してあってもそのポートを指定することはできません。


## 現在の制約

- 通信が遅延したり良くないことが起きるとゲームがフレーズすることがあります。


## 配布元

https://github.com/progre/junowen
