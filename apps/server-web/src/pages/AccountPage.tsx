// 帳號自助頁的殼（server-web-console 導覽）。使用者、PAT、Web session、裝置的
// 讀取與 mutation 在 account／PAT／device 面（後續 knife）補上。
export function AccountPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">帳號</h1>
      <p className="mt-2 text-muted-foreground">
        個人資料、存取權杖、Web session 與裝置。
      </p>
    </div>
  );
}
