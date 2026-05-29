import { createApp } from "vue"
import App from "./App.vue"
import router from "./router"
import { initApp } from "./services/api/settings"

async function bootstrap() {
  try {
    await initApp()
  } catch {
    // app will retry DB-backed calls where needed
  }
  createApp(App).use(router).mount("#app")
}

bootstrap()
