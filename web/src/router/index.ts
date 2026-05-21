import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
      meta: { public: true },
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { public: true },
    },
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/HomeView.vue'),
    },
    {
      path: '/libraries',
      name: 'libraries',
      component: () => import('@/views/LibrariesView.vue'),
    },
    {
      path: '/libraries/add',
      name: 'add-library',
      component: () => import('@/views/AddLibraryView.vue'),
    },
    {
      path: '/books',
      name: 'books',
      component: () => import('@/views/BooksView.vue'),
    },
    {
      path: '/books/:id',
      name: 'book-detail',
      component: () => import('@/views/BookDetailView.vue'),
    },
    {
      path: '/authors',
      name: 'authors',
      component: () => import('@/views/AuthorsView.vue'),
    },
    {
      path: '/authors/:id',
      name: 'author-books',
      component: () => import('@/views/AuthorBooksView.vue'),
    },
    {
      path: '/series',
      name: 'series',
      component: () => import('@/views/SeriesView.vue'),
    },
    {
      path: '/series/:id',
      name: 'series-books',
      component: () => import('@/views/SeriesBooksView.vue'),
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('@/views/SearchView.vue'),
    },
    {
      path: '/bookmarks',
      name: 'bookmarks',
      component: () => import('@/views/BookmarksView.vue'),
    },
    {
      path: '/collections',
      name: 'collections',
      component: () => import('@/views/CollectionsView.vue'),
    },
    {
      path: '/collections/:id',
      name: 'collection-detail',
      component: () => import('@/views/CollectionDetailView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: () => import('@/views/NotFoundView.vue'),
    },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()

  if (!auth.user && auth.isAuthenticated) {
    await auth.fetchMe()
  }

  if (!to.meta.public && !auth.isAuthenticated) {
    return { name: 'login' }
  }

  if (to.meta.public && auth.isAuthenticated && to.name !== 'setup') {
    return { name: 'home' }
  }
})

export default router
