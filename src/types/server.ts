export interface InputArg {
  name: string
  description: string
  class: 'Text' | 'Select' | 'FilePath' | 'DirectoryPath'
  multiplicity: 'Single' | 'Multiple'
  value: string[]
}

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
  inputArg: InputArg
}

export interface ServerCardData extends BaseServerCardData {
  publishDate: Date
}

export interface RawServerCardData extends BaseServerCardData {
  publishDate: string;
}
export type InstallStatus = 'install' | 'installing' | 'installed' | 'uninstall'
