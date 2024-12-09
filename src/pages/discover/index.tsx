import { ServerCard } from "@/components/ServerCard";
import { RawServerCardData, ServerCardData } from '@/types/server';
import { parseDate } from "@/utils/parseDate";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export default function DiscoverPage() {
  const [serverCards, setServerCards] = useState<ServerCardData[]>([]);

  useEffect(() => {
    const fetchServers = async () => {
      const rawServerCards: RawServerCardData[] = await invoke("get_servers");
      const processedCards = rawServerCards.map((card) => ({
        ...card,
        publishDate: parseDate(card.publishDate),
      }));
      setServerCards(processedCards);
      console.log("processedCards", processedCards);
    };

    fetchServers();
  }, []);

  return (
    <div className="container mx-auto p-8">
      <h1 className="text-3xl font-bold mb-8 text-center">Discover MCPHub Servers</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
        {serverCards.map((card, index) => (
          <ServerCard
            key={index}
            {...card}
          />
        ))}
      </div>
    </div>
  )
}
