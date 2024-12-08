import { Tag } from "@/components/Tag"
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { InstallStatus, ServerCardData } from '@/types/server'
import { getRelativeTime } from '@/utils/getRelativeTime'
import { motion } from 'framer-motion'
import { Check, Download, Loader2, Star } from 'lucide-react'
import { useState } from 'react'

type ServerCardProps = ServerCardData

export function ServerCard({
    title,
    description,
    creator,
    logoUrl,
    publishDate,
    rating,
    tags,
    isInstalled
}: ServerCardProps) {
    const [isHovered, setIsHovered] = useState(false)
    const [installStatus, setInstallStatus] = useState<InstallStatus>(isInstalled ? 'installed' : 'install')
    const relativeTime = getRelativeTime(publishDate)

    const handleInstall = () => {
        if (installStatus === 'install') {
            setInstallStatus('installing')
            setTimeout(() => setInstallStatus('installed'), 2000) // Simulate installation
        } else if (installStatus === 'installed') {
            setInstallStatus('install') // Uninstall
        }
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
                            <AvatarImage src={logoUrl} alt={creator} />
                            <AvatarFallback>{creator[0]}</AvatarFallback>
                        </Avatar>
                        <div>
                            <h3 className="font-semibold text-base leading-none mb-1">{title}</h3>
                            <p className="text-sm text-muted-foreground">{creator}</p>
                        </div>
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
        </motion.div>
    )
}
