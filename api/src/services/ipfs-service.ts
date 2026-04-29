import { Readable } from "stream";
import FormData from "form-data";
import { env } from "../config/env";
import { logger } from "../config/logger";

export interface IpfsUploadResult {
  cid: string;
  gatewayUrl: string;
  size: number;
}

export class IpfsService {
  private readonly pinataApiUrl = "https://api.pinata.cloud/pinning/pinFileToIPFS";
  private readonly pinataJwt: string;
  private readonly gatewayBase: string;

  constructor() {
    this.pinataJwt = env.pinataJwt;
    this.gatewayBase = env.ipfsGateway;
  }

  async uploadFile(
    fileBuffer: Buffer,
    fileName: string,
    mimeType: string,
  ): Promise<IpfsUploadResult> {
    const form = new FormData();
    form.append("file", Readable.from(fileBuffer), {
      filename: fileName,
      contentType: mimeType,
      knownLength: fileBuffer.length,
    });
    form.append(
      "pinataMetadata",
      JSON.stringify({ name: fileName }),
      { contentType: "application/json" },
    );

    const headers: Record<string, string> = {
      ...(form.getHeaders() as Record<string, string>),
      Authorization: `Bearer ${this.pinataJwt}`,
    };

    const response = await fetch(this.pinataApiUrl, {
      method: "POST",
      headers,
      body: form as unknown as BodyInit,
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`IPFS upload failed (${response.status}): ${text}`);
    }

    const json = (await response.json()) as { IpfsHash: string; PinSize: number };
    const cid = json.IpfsHash;

    return {
      cid,
      gatewayUrl: `${this.gatewayBase}/ipfs/${cid}`,
      size: json.PinSize,
    };
  }

  gatewayUrl(cid: string): string {
    return `${this.gatewayBase}/ipfs/${cid}`;
  }
}

export const ipfsService = new IpfsService();
