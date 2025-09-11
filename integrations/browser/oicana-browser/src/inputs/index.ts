import {JsonInputDefinition} from "./JsonInput";
import {BlobInputDefinition, BlobWithMetadata} from "./BlobInput";

export {JsonInputDefinition, BlobInputDefinition, BlobWithMetadata};

export interface Inputs {
    json: JsonInputDefinition[],
    blob: BlobInputDefinition[]
}
