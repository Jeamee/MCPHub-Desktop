import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import React from "react";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";

function useAutoUpdater() {
  React.useEffect(() => {
    async function checkForUpdates() {
      try {
        const update = await check();
        if (update) {
          console.log(`Update available: ${update.version}`);

          await update.downloadAndInstall((event) => {
            switch (event.event) {
              case 'Started':
                break;
              case 'Progress':
                break;
              case 'Finished':
                break;
            }
          });

          await relaunch();
        }
      } catch (error) {
        console.error('Update failed:', error);
      }
    }

    checkForUpdates();
  }, []);
}

export default function App() {
  useAutoUpdater();
  return <RouterProvider router={router} />
}
