interface BaseServerCardData {
  title: string
  description: string
  creator: string
  logoUrl: string
  rating: number
  tags: string[]
  isInstalled: boolean
}

export interface ServerCardData extends BaseServerCardData {
  publishDate: Date
}

export interface RawServerCardData extends BaseServerCardData {
  publishDate: string;
}
export type InstallStatus = 'install' | 'installing' | 'installed' | 'uninstall'
