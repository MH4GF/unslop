# auto-fix basics

prh: ワーカー はワーカー に直す。

半角カナ: アイウとガギを全角化する。

ZWSP: あいに紛れる zero width space は削除する。

NFD: ボケットはポケットへ NFC 正規化する。

制御文字: helloworld の BEL は削除する。

mixed-period: 末尾の ASCII period は句点に置換する。

redundant: これは省略することが可能である。

abusage: ファイルは書きづらいので注意。

abusage 2: try で例外を捕捉する書き方を直す。

ja-spacing: あれはダメで、JTF 標準と書く。
