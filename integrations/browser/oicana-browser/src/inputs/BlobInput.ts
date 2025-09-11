export interface BlobWithMetadata {
    bytes: Uint8Array,
    meta?: {
        image_format?: string,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        [key: string]: any;
    }
}

export interface BlobInputDefinition {
    key: string,
    default: {
        file: string,
        meta?: {
            image_format?: string,
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            [key: string]: any;
        }
    }
}
