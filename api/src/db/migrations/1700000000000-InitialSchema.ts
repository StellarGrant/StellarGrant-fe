import { MigrationInterface, QueryRunner } from "typeorm";

// Baseline migration: schema already exists in production (created via synchronize).
// This migration is intentionally a no-op so that future migrations can be tracked
// from this baseline without re-creating existing tables.
export class InitialSchema1700000000000 implements MigrationInterface {
  name = "InitialSchema1700000000000";

  public async up(_queryRunner: QueryRunner): Promise<void> {
    // no-op: schema was bootstrapped via TypeORM synchronize
  }

  public async down(_queryRunner: QueryRunner): Promise<void> {
    // no-op
  }
}
