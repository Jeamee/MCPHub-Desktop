import { createHashRouter } from "react-router-dom"

import { Layout } from "@/components/Layout"
import DiscoverPage from "@/pages/discover"
import MyServersPage from "@/pages/my"
import WelcomePage from "@/pages/welcome"

export const router = createHashRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      {
        index: true,
        element: <WelcomePage />
      },
      {
        path: "discover",
        element: <DiscoverPage />
      },
      {
        path: "installed",
        element: <MyServersPage />
      },
    ]
  }
])