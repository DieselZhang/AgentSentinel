import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'run-list',
      component: () => import('@/views/RunList.vue'),
    },
    {
      path: '/runs/:id',
      name: 'run-detail',
      component: () => import('@/views/RunDetail.vue'),
    },
    {
      path: '/compare',
      name: 'compare',
      component: () => import('@/views/CompareView.vue'),
    },
    {
      path: '/upload',
      name: 'upload',
      component: () => import('@/views/UploadView.vue'),
    },
  ],
})

export default router
