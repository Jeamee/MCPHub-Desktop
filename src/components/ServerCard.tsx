import { Tag } from "@/components/Tag"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { InstallStatus, ServerCardData } from '@/types/server'
import { getRelativeTime } from '@/utils/getRelativeTime'
import { invoke } from "@tauri-apps/api/core"
import { motion } from 'framer-motion'
import { Check, Download, Loader2, Settings, Star } from 'lucide-react'
import { useState } from 'react'
import { ConfigModal } from "./ConfigModal"

type ServerCardProps = ServerCardData

export function ServerCard({
    id,
    title,
    description,
    creator,
    logoUrl,
    publishDate,
    rating,
    tags,
    isInstalled,
    env,
    guide
}: ServerCardProps) {
    const [isHovered, setIsHovered] = useState(false)
    const [isConfigModalOpen, setIsConfigModalOpen] = useState(false)
    const [installStatus, setInstallStatus] = useState<InstallStatus>(isInstalled ? 'installed' : 'install')
    const relativeTime = getRelativeTime(publishDate)

    const handleConfigSave = async (config: Record<string, string>) => {
        console.log('Saved config:', config)
        setInstallStatus('installing')
        await invoke('update_server', { serverId: id, env: config })
        setInstallStatus('installed')
    }

    const handleInstall = async () => {
        if (Object.keys(env).length === 0) {
            setIsConfigModalOpen(false);
            if (installStatus === 'install') {
                setInstallStatus('installing');
                await invoke('install_server', { serverId: id });
                setInstallStatus('installed');
            } else if (installStatus === 'installed') {
                await invoke('uninstall_server', { serverId: id });
                setInstallStatus('install');
            }
        } else {
            if (installStatus === 'install') {
                setIsConfigModalOpen(true);
            };
        };
    }

    const getButtonContent = () => {
        switch (installStatus) {
            case 'install':
                return (
                    <>
                        <Download className="mr-2 h-4 w-4" />
                        Install
                    </>
                )
            case 'installing':
                return (
                    <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Installing...
                    </>
                )
            case 'installed':
                return isHovered ? (
                    <>
                        <Download className="mr-2 h-4 w-4" />
                        Uninstall
                    </>
                ) : (
                    <>
                        <Check className="mr-2 h-4 w-4" />
                        Installed
                    </>
                )
            default:
                return 'Install'
        }
    }

    return (
        <motion.div
            whileHover={{ scale: 1.05 }}
            transition={{ type: "spring", stiffness: 300 }}
            onHoverStart={() => setIsHovered(true)}
            onHoverEnd={() => setIsHovered(false)}
        >
            <Card className="w-full max-w-sm overflow-hidden bg-gradient-to-br from-white to-gray-100 dark:from-gray-800 dark:to-gray-900 shadow-lg">
                <CardContent className="p-4">
                    <div className="flex items-center space-x-3 mb-3">
                        <Avatar className="h-10 w-10">
                            <AvatarImage src={logoUrl} alt={title} />
                            <AvatarFallback>{title[0]}</AvatarFallback>
                        </Avatar>
                        <div>
                            <h3 className="font-semibold text-base leading-none mb-1">{title}</h3>
                            <p className="text-sm text-muted-foreground">{creator}</p>
                        </div>
                        {installStatus === 'installed' && Object.keys(env).length > 0 && (
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => setIsConfigModalOpen(true)}
                                className="ml-auto"
                            >
                                <Settings className="h-4 w-4" />
                            </Button>
                        )}
                    </div>
                    <p className="text-sm text-muted-foreground mb-3">{description}</p>
                    <div className="flex flex-wrap mb-3">
                        {tags.map((tag, index) => (
                            <Tag key={index} name={tag} />
                        ))}
                    </div>
                    <div className="flex justify-between items-center mb-3">
                        <div className="flex">
                            {[...Array(5)].map((_, i) => (
                                <Star
                                    key={i}
                                    className={`w-4 h-4 ${i < rating
                                        ? "text-yellow-400 fill-yellow-400"
                                        : "text-gray-300 dark:text-gray-600"
                                        } ${isHovered ? 'animate-pulse' : ''}`}
                                />
                            ))}
                        </div>
                        <motion.p
                            className="text-xs text-muted-foreground"
                            initial={{ opacity: 0.6 }}
                            animate={{ opacity: isHovered ? 1 : 0.6 }}
                        >
                            {relativeTime}
                        </motion.p>
                    </div>
                    <Button
                        className="w-full"
                        variant={installStatus === 'installed' ? 'secondary' : 'default'}
                        onClick={handleInstall}
                        disabled={installStatus === 'installing'}
                    >
                        {getButtonContent()}
                    </Button>
                </CardContent>
            </Card>
            <ConfigModal
                isOpen={isConfigModalOpen}
                onClose={() => setIsConfigModalOpen(false)}
                env={env}
                guide={guide}
                onSave={handleConfigSave}
            />
        </motion.div>
    )
}