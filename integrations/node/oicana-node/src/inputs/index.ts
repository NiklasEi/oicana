import { BlobInputDefinition, BlobWithMetadata } from './BlobInput.js';
import { JsonInputDefinition } from './JsonInput.js';

export { JsonInputDefinition, BlobInputDefinition, BlobWithMetadata };

export interface Inputs {
  json: JsonInputDefinition[];
  blob: BlobInputDefinition[];
}
