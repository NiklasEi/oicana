import { BlobInput, BlobInputDefinition, BlobMetadata } from './BlobInput.js';
import { JsonInputDefinition } from './JsonInput.js';

export { JsonInputDefinition, BlobInput, BlobInputDefinition, BlobMetadata };

export interface Inputs {
  json: JsonInputDefinition[];
  blob: BlobInputDefinition[];
}
