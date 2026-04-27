import { Column, Entity, OneToMany, PrimaryColumn, UpdateDateColumn } from "typeorm";
import { Milestone } from "./Milestone";
import { MilestoneProof } from "./MilestoneProof";
import { GrantReviewer } from "./GrantReviewer";

@Entity({ name: "grants" })
export class Grant {
  @PrimaryColumn({ type: "int" })
  id!: number;

  @Column({ type: "varchar", length: 200 })
  title!: string;

  @Column({ type: "varchar", length: 30 })
  status!: string;

  @Column({ type: "varchar", length: 120 })
  recipient!: string;

  @Column({ type: "varchar", length: 60 })
  totalAmount!: string;

  /**
   * Comma-separated tags stored as a simple text column for broad DB compatibility.
   * The route layer splits / joins this value when reading / writing.
   */
  @Column({ type: "text", nullable: true })
  tags!: string | null;

  @Column({ type: "simple-json", nullable: true })
  localizedMetadata!: Record<string, { title?: string; description?: string }> | null;

  @UpdateDateColumn()
  updatedAt!: Date;

  @OneToMany(() => MilestoneProof, (proof) => proof.grant)
  proofs!: MilestoneProof[];

  @OneToMany(() => Milestone, (milestone) => milestone.grant)
  milestones!: Milestone[];
}
