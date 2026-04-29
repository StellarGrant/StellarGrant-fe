import Link from "next/link";

export function AppHeader() {
  return (
    <header className="border-b">
      <div className="container mx-auto px-4 py-4 flex items-center justify-between">
        <div className="flex items-center gap-8">
          <Link href="/" className="text-xl font-bold">StellarGrants</Link>
          <nav className="flex gap-4">
            <Link href="/grants">Explore</Link>
            <Link href="/grants/create">Create</Link>
            <Link href="/profile">Profile</Link>
          </nav>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-sm px-2 py-1 bg-yellow-100 rounded">Testnet</span>
          {/* WalletConnect component will be rendered here */}
        </div>
      </div>
    </header>
  );
}
