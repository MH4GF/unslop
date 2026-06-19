# auto-fix basics

prh: worker は ワーカー に直す。

半角カナ: ｱｲｳ と ｶﾞｷﾞ を全角化する。

ZWSP: あ​い に紛れる zero width space は削除する。

NFD: ボケット は ポケット へ NFC 正規化する。

制御文字: helloworld の BEL は削除する。

mixed-period: 末尾の ASCII period は句点に置換する.

redundant: これは省略することが可能である。

abusage: ファイルは書きずらいので注意。

abusage 2: try で例外を補足する書き方を直す。

ja-spacing: あれは ダメ で、JTF標準 と書く。
