import { Outlet } from "react-router-dom"
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/AppSidebar"

export function Layout() {
  return (
    <SidebarProvider>
      <div className="flex h-screen w-full overflow-hidden">
        <AppSidebar />
        <main className="flex-1 min-w-0 overflow-auto">
          <div className="p-4">
            <SidebarTrigger />
          </div>
          <div className="flex-1 min-w-0 px-6">
            <Outlet />
          </div>
        </main>
      </div>
    </SidebarProvider>
  )
}