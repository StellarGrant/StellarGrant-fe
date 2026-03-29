import { DataSource, Repository } from "typeorm";
import { Grant } from "../entities/Grant";
import { SorobanContractClient } from "../soroban/types";

export class GrantSyncService {
  private readonly grantRepo: Repository<Grant>;

  constructor(
    private readonly dataSource: DataSource,
    private readonly sorobanClient: SorobanContractClient,
  ) {
    this.grantRepo = this.dataSource.getRepository(Grant);
  }

  async syncAllGrants(): Promise<void> {
    const grants = await this.sorobanClient.fetchGrants();
    for (const grant of grants) {
      await this.grantRepo.save(grant);
    }
  }

  async syncGrant(id: number): Promise<void> {
    const grant = await this.sorobanClient.fetchGrantById(id);
    if (!grant) return;
    await this.grantRepo.save(grant);
  }
}
