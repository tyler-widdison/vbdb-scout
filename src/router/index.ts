import { createRouter, createWebHashHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    {
      path: "/seasons",
      name: "seasons",
      component: () => import("../views/SeasonsView.vue"),
    },
    {
      path: "/video-player",
      name: "video-player",
      component: () => import("../views/VideoWindowView.vue"),
    },
  ],
});

export default router;
