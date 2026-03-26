import { SorobanContractClient, SorobanGrant } from "./types";

const mockGrants: SorobanGrant[] = [
  {
    id: 1,
    title: "Open Source Grants Q2",
    status: "active",
    recipient: "GBRPYHIL2C2WBO36G6UIGR2PA4M3TQ7VOY3RTMAL4LRRA67ZOHQ65SZD",
    totalAmount: "250000000",
  },
  {
    id: 2,
    title: "Climate Data Tools",
    status: "review",
    recipient: "GCBQ6JQXQTVV7T7OUVPR4Q6PGACCUAKS6S2YDG3YQYQYRR2NJB5A6NAA",
    totalAmount: "100000000",
  },
];

export class MockSorobanContractClient implements SorobanContractClient {
  async fetchGrants(): Promise<SorobanGrant[]> {
    return mockGrants;
  }

  async fetchGrantById(id: number): Promise<SorobanGrant | null> {
    return mockGrants.find((grant) => grant.id === id) ?? null;
  }
}
