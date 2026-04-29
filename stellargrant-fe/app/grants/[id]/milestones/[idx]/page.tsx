/**
 * Milestone Detail Page
 *
 * Shows a single milestone's submitted proof, vote count, reviewer list,
 * and action buttons (Submit / Approve / Reject) depending on connected wallet role.
 */

export default async function MilestoneDetailPage({ params }: { params: Promise<{
    id: string;
    idx: string;
  }> }) {
  const { id, idx } = await params;

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-3xl font-bold mb-6">
        Milestone {idx} - Grant #{id}
      </h1>
      {/* Milestone detail + vote will be implemented here */}
    </div>
  );
}
