import { ContributorScore, SorobanContractClient, SorobanContractEvent, SorobanGrant } from "./types";

const toIsoDeadline = (daysFromNow: number) => {
  const date = new Date();
  date.setUTCHours(12, 0, 0, 0);
  date.setUTCDate(date.getUTCDate() + daysFromNow);
  return date.toISOString();
};

const mockGrants: SorobanGrant[] = [
  {
    id: 1,
    title: "Open Source Grants Q2",
    status: "active",
    recipient: "GBRPYHIL2C2WBO36G6UIGR2PA4M3TQ7VOY3RTMAL4LRRA67ZOHQ65SZD",
    totalAmount: "250000000",
    tags: "open-source,web3,tooling",
    localizedMetadata: {
      en: { title: "Open Source Grants Q2", description: "Supporting the best open-source projects." },
      es: { title: "Subvenciones de Código Abierto Q2", description: "Apoyando los mejores proyectos de código abierto." },
    },
    milestones: [
      {
        idx: 0,
        title: "Architecture review",
        description: "Finalize the implementation plan and publish the design brief.",
        deadline: toIsoDeadline(7),
      },
      {
        idx: 1,
        title: "Prototype delivery",
        description: "Ship the first contributor-ready prototype for evaluation.",
        deadline: toIsoDeadline(-2),
      },
    ],
  },
  {
    id: 2,
    title: "Climate Data Tools",
    status: "review",
    recipient: "GCBQ6JQXQTVV7T7OUVPR4Q6PGACCUAKS6S2YDG3YQYQYRR2NJB5A6NAA",
    totalAmount: "100000000",
    tags: "climate,data,open-source",
    localizedMetadata: {
      en: { title: "Climate Data Tools", description: "Tools for measuring climate impact." },
    },
    milestones: [
      {
        idx: 0,
        title: "Dataset ingestion",
        description: "Import the first climate dataset and validate refresh jobs.",
        deadline: toIsoDeadline(3),
      },
    ],
  },
  {
    id: 3,
    title: "DeFi Infrastructure Fund",
    status: "active",
    recipient: "GDZAPKZFP3PVPRMDG6WQVIMZLQ5J3FZGQ27BFLDL3YQSM6L7LS6AXEX",
    totalAmount: "500000000",
    tags: "defi,web3,infrastructure",
    milestones: [
      {
        idx: 0,
        title: "Validator integration",
        description: "Connect the new validator pipeline and complete smoke tests.",
        deadline: toIsoDeadline(1),
      },
    ],
  },
  {
    id: 4,
    title: "Community Education Initiative",
    status: "pending",
    recipient: "GAV3TIZZ7DRCCMUVKZRQXELRTJFMXQT4XJFNV5BYMNOFXWXZA5MGDVEV",
    totalAmount: "75000000",
    tags: "education,community",
    milestones: [
      {
        idx: 0,
        title: "Curriculum planning",
        description: "Draft the first training curriculum and secure mentors.",
        deadline: toIsoDeadline(14),
      },
    ],
  },
];

export class MockSorobanContractClient implements SorobanContractClient {
  async fetchGrants(): Promise<SorobanGrant[]> {
    return mockGrants;
  }

  async fetchGrantById(id: number): Promise<SorobanGrant | null> {
    return mockGrants.find((grant) => grant.id === id) ?? null;
  }

  async fetchContributorScore(address: string): Promise<ContributorScore | null> {
    // Basic mock logic: return a consistent score for known mock addresses
    const knownAddresses = mockGrants.map((g) => g.recipient);
    if (!knownAddresses.includes(address)) return null;

    return {
      address,
      reputation: 100, // Dummy fixed reputation for mocks
      totalEarned: "1000000000",
    };
  }

  async fetchEvents(fromLedger: number, toLedger: number): Promise<SorobanContractEvent[]> {
    // Return synthetic events so the reconciliation task has something to process in dev/test.
    const events: SorobanContractEvent[] = [];
    for (const grant of mockGrants) {
      const ledger = fromLedger + (grant.id % Math.max(1, toLedger - fromLedger + 1));
      events.push({
        ledger,
        id: `${ledger}-mock-${grant.id}-0`,
        type: "grant_created",
        grantId: grant.id,
        actorAddress: grant.recipient,
        data: { title: grant.title, totalAmount: grant.totalAmount, status: grant.status },
      });
    }
    return events;
  }

  async getLatestLedger(): Promise<number> {
    // Simulate a slowly advancing ledger (5-second Stellar close time).
    return Math.floor(Date.now() / 5000);
  }
}
