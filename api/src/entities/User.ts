import { Column, Entity, PrimaryColumn } from "typeorm";

@Entity()
export class User {
  @PrimaryColumn({ type: "varchar", length: 56 })
  stellarAddress!: string;

  @Column({ type: "varchar", nullable: true })
  githubId?: string;

  @Column({ type: "varchar", nullable: true })
  githubUsername?: string;

  @Column({ type: "varchar", nullable: true })
  twitterId?: string;

  @Column({ type: "varchar", nullable: true })
  twitterUsername?: string;
}
