import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { open as pathSelect } from '@tauri-apps/plugin-dialog';
import { Label } from "@/components/ui/label";
import type { InputArg } from "@/types/server";
import { open as urlOpen } from '@tauri-apps/plugin-shell';
import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { Button } from "@/components/ui/button";
import { FolderOpen, Plus } from "lucide-react";

interface ConfigModalProps {
    isOpen: boolean
    onClose: () => void
    env: Record<string, string>
    guide: string
    inputArg: InputArg
    onSave: (config: Record<string, string>, args: string[]) => void
}

export function ConfigModal({ isOpen, onClose, env, guide, inputArg, onSave }: ConfigModalProps) {
    const [config, setConfig] = useState<Record<string, string>>({})

    const handleInputChange = (key: string, value: string) => {
        setConfig(prev => ({ ...prev, [key]: value }))
    }
    const [argValues, setArgValues] = useState<string[]>(inputArg.value);

    const handleArgValueChange = (index: number, value: string) => {
        const newValues = [...argValues];
        newValues[index] = value;
        setArgValues(newValues);
    };

    const handleAddValue = () => {
        setArgValues([...argValues, ""]);
    };

    
    const handleFileSelect = async (index: number) => {
        let selected: string | null;
        if (inputArg.class === "DirectoryPath") {
            selected = await pathSelect({
                directory: true,
                multiple: false
            });
        } else {
            selected = await pathSelect({
                multiple: false,
            });
        }
        
        if (selected) {
            handleArgValueChange(index, selected as string);
        }
    };

    const handleSave = () => {
        onSave(config, argValues)
        onClose()
    }

    return (
        <Dialog open={isOpen} onOpenChange={onClose}>
            <DialogContent className="sm:max-w-[600px] p-0 gap-0 bg-gradient-to-br from-white to-gray-100 dark:from-gray-800 dark:to-gray-900">
                <DialogHeader className="p-6 pb-4 space-y-4">
                    <div className="flex items-center justify-between">
                        <DialogTitle className="text-2xl font-semibold">Configuration</DialogTitle>
                    </div>
                    {guide && (
                        <div className="bg-muted/50 rounded-lg p-4 prose dark:prose-invert max-w-none">
                            <ReactMarkdown
                                components={{
                                    a: ({ node, ...props }) => (
                                        <a
                                            {...props}
                                            onClick={(event: React.MouseEvent) => {
                                                event.preventDefault();
                                                if (props.href) {
                                                    urlOpen(props.href);
                                                }
                                            }}
                                            className="text-blue-500 hover:underline cursor-pointer"
                                        />
                                    )
                                }}
                            >
                                {guide}
                            </ReactMarkdown>
                        </div>
                    )}
                </DialogHeader>
                <div className="px-6 py-4 border-y">
                    <div className="space-y-4">
                        {Object.keys(env).map((key) => (
                            <div key={key} className="flex flex-col space-y-2">
                                <Label htmlFor={key} className="font-medium">
                                    {key}
                                </Label>
                                <Input
                                    id={key}
                                    placeholder={`Enter your ${key.toLowerCase()}`}
                                    value={config[key] || env[key] || ''}
                                    onChange={(e) => handleInputChange(key, e.target.value)}
                                />
                            </div>
                        ))}
                        {
                            inputArg.name && (
                                <div className="space-y-4">
                                    <div className="flex flex-col space-y-2">
                                        <Label className="text-lg font-semibold">
                                            {inputArg.name}
                                        </Label>
                                        
                                        {argValues.map((value, index) => (
                                            <div key={index} className="flex gap-2">
                                                {inputArg.class === "Text" ? (
                                                    <Input
                                                        value={value}
                                                        onChange={(e) => handleArgValueChange(index, e.target.value)}
                                                        placeholder={`Enter ${inputArg.name.toLowerCase()}`}
                                                    />
                                                ) : (
                                                    <div className="flex w-full gap-2">
                                                        <Input
                                                            value={value}
                                                            readOnly
                                                            placeholder={`Select ${inputArg.class === "FilePath" ? "file" : "directory"}`}
                                                        />
                                                        <Button
                                                            type="button"
                                                            variant="outline"
                                                            onClick={() => handleFileSelect(index)}
                                                        >
                                                            <FolderOpen className="h-4 w-4" />
                                                        </Button>
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                                        
                                        {inputArg.multiplicity === "Multiple" && (
                                            <Button
                                                type="button"
                                                variant="outline"
                                                onClick={handleAddValue}
                                                className="w-full mt-2"
                                            >
                                                <Plus className="h-4 w-4 mr-2" />
                                                Add {inputArg.name}
                                            </Button>
                                        )}
                                    </div>
                                    
                                    {inputArg.description && (
                                        <p className="text-sm text-muted-foreground">
                                            {inputArg.description}
                                        </p>
                                    )}
                                </div>
                            )}
                    </div>
                </div>
                <DialogFooter className="p-6 pt-4">
                    <Button
                        type="submit"
                        onClick={handleSave}
                        className="w-full sm:w-auto"
                    >
                        Save changes
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
