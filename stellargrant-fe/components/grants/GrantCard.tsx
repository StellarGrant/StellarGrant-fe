/**
 * GrantCard Component
 * 
 * Compact card for grant listing pages. Shows title, status badge,
 * funding progress, deadline, and token.
 */

interface GrantCardProps {
  grant: {
    id: string;
    title: string;
    status: string;
    funded: number;
    target: number;
    deadline: Date;
    token: string;
  };
  onClick?: () => void;
  showOwner?: boolean;
  compact?: boolean;
}

export function GrantCard({ grant, onClick: _onClick, showOwner: _showOwner = false, compact: _compact = false }: GrantCardProps) {
  // TODO: Implement GrantCard component
  return (
    <div className="border rounded-lg p-4 cursor-pointer hover:shadow-md transition-shadow">
      <h3 className="text-xl font-semibold">{grant.title}</h3>
      {/* Grant card content will be implemented here */}
    </div>
  );
}
