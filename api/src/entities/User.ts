import { Entity, PrimaryGeneratedColumn, Column, CreateDateColumn, UpdateDateColumn } from "typeorm";

@Entity({ name: "users" })
export class User {
  @PrimaryGeneratedColumn()
  id!: number;


  @Column({ type: "varchar", length: 120, unique: true })
  email!: string;

  @Column({ type: "varchar", length: 56, unique: true })
  stellarAddress!: string;

  @Column({ type: "boolean", default: true })
  notifyMilestoneApproved!: boolean;

  @Column({ type: "boolean", default: true })
  notifyMilestoneSubmitted!: boolean;

  @CreateDateColumn()
  createdAt!: Date;

  @UpdateDateColumn()
  updatedAt!: Date;
}
