/**
 * Fund Grant Page
 *
 * Dedicated funding flow. Lets any address deposit XLM or USDC
 * into a grant's escrow.
 */

export default async function FundGrantPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-3xl font-bold mb-6">Fund Grant #{id}</h1>
      {/* Fund grant flow will be implemented here */}
    </div>
  );
}
