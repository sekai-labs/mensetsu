export type Browser = 'chrome' | 'firefox';

export interface BuildOptions {
  quiet?: boolean;
  browser?: Browser | null;
}

export interface BuildResult {
  distDir: string;
  outputs: Partial<Record<Browser, string>>;
  elapsed: number;
}

export declare const BROWSERS: readonly Browser[];

export declare function build(options?: BuildOptions): Promise<BuildResult>;
