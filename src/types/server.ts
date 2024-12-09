interface BaseServerCardData {
  id: string
  title: string
  description: string
  creator: string
  logoUrl: string
  rating: number
  tags: string[]
  isInstalled: boolean,
  env: Record<string, string>
  guide: string
}

export interface ServerCardData extends BaseServerCardData {
  publishDate: Date
}

export interface RawServerCardData extends BaseServerCardData {
  publishDate: string;
}
export type InstallStatus = 'install' | 'installing' | 'installed' | 'uninstall'
