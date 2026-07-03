<!DOCTYPE html>
<html lang="en">
<!--begin::Head-->
<head>
    <base href="{{ url('/') }}/" />
    <title>@yield('title') — {{ $appSettings['site_name'] ?? 'StarterTemp' }}</title>
    <meta charset="utf-8" />
    <meta name="description" content="{{ $appSettings['site_name'] ?? 'StarterTemp' }} — Admin Dashboard" />
    <meta name="author" content="Rendy Irawan" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta property="og:locale" content="id_ID" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{{ $appSettings['site_name'] ?? 'StarterTemp' }} — Dashboard" />
    <meta property="og:url" content="{{ url()->current() }}" />
    <meta property="og:site_name" content="{{ $appSettings['site_name'] ?? 'StarterTemp' }}" />
    <link rel="canonical" href="{{ url()->current() }}" />
    @php
        $siteLogo = $appSettings['site_logo'] ?? 'base-logo.png';
        $siteFont = $appSettings['site_font'] ?? 'Plus Jakarta Sans';
        $siteName = $appSettings['site_name'] ?? 'StarterTemp';
    @endphp
    <link rel="shortcut icon" href="{{ asset('assets/media/logos/' . $siteLogo) }}" />
    <!--begin::Fonts-->
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family={{ str_replace(' ', '+', $siteFont) }}:wght@300;400;500;600;700;800&display=swap" />
    <!--end::Fonts-->
    <!--begin::Vendor Stylesheets-->
    <link href="{{ asset('assets/plugins/custom/fullcalendar/fullcalendar.bundle.css') }}" rel="stylesheet" type="text/css" />
    <link href="{{ asset('assets/plugins/custom/datatables/datatables.bundle.css') }}" rel="stylesheet" type="text/css" />
    <!--end::Vendor Stylesheets-->
    <!--begin::Global Stylesheets Bundle(mandatory for all pages)-->
    <link href="{{ asset('assets/plugins/global/plugins.bundle.css') }}" rel="stylesheet" type="text/css" />
    <link href="{{ asset('assets/css/style.bundle.css') }}" rel="stylesheet" type="text/css" />
    <!--end::Global Stylesheets Bundle-->
    <style>
        :root {
            --bs-font-sans-serif: '{{ $siteFont }}', sans-serif;
            --bs-body-font-family: '{{ $siteFont }}', sans-serif;
        }
        body {
            font-family: '{{ $siteFont }}', sans-serif !important;
        }
        h1, h2, h3, h4, h5, h6, .h1, .h2, .h3, .h4, .h5, .h6 {
            font-family: '{{ $siteFont }}', sans-serif !important;
        }
        /* Sidebar overlay for mobile */
        .sidebar-overlay {
            display: none;
            position: fixed;
            top: 0; left: 0; right: 0; bottom: 0;
            background: rgba(0,0,0,0.4);
            z-index: 104;
        }
        .sidebar-overlay.active { display: block; }
        /* Custom sidebar styles for Demo 11 */
        #kt_app_sidebar {
            position: fixed;
            left: 0;
            top: 0;
            bottom: 0;
            width: 280px;
            z-index: 105;
            background: #fff;
            border-right: 1px solid var(--bs-gray-200);
            transform: translateX(-100%);
            transition: transform 0.3s ease;
            overflow-y: auto;
        }
        #kt_app_sidebar.active { transform: translateX(0); }
        [data-bs-theme="dark"] #kt_app_sidebar {
            background: #1e1e2d;
            border-right-color: rgba(255,255,255,0.07);
        }
    </style>
    <script>
        if (window.top != window.self) { window.top.location.replace(window.self.location.href); }
    </script>
    @stack('stylesheets')
</head>
<!--end::Head-->
<!--begin::Body-->
<body id="kt_body" class="header-fixed header-tablet-and-mobile-fixed toolbar-enabled">
    <!--begin::Theme mode setup on page load-->
    <script>
        var defaultThemeMode = "light";
        var themeMode;
        if (document.documentElement) {
            if (document.documentElement.hasAttribute("data-bs-theme-mode")) {
                themeMode = document.documentElement.getAttribute("data-bs-theme-mode");
            } else {
                if (localStorage.getItem("data-bs-theme") !== null) {
                    themeMode = localStorage.getItem("data-bs-theme");
                } else {
                    themeMode = defaultThemeMode;
                }
            }
            if (themeMode === "system") {
                themeMode = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
            }
            document.documentElement.setAttribute("data-bs-theme", themeMode);
        }
    </script>
    <!--end::Theme mode setup on page load-->
    <!--begin::Main-->
    <!--begin::Root-->
    <div class="d-flex flex-column flex-root">
        <!--begin::Page-->
        <div class="page d-flex flex-row flex-column-fluid">
            <!--begin::Wrapper-->
            <div class="wrapper d-flex flex-column flex-row-fluid" id="kt_wrapper">
                <!--begin::Header-->
                <div id="kt_header" class="header" data-kt-sticky="true" data-kt-sticky-name="header" data-kt-sticky-offset="{default: '200px', lg: '300px'}">
                    <!--begin::Container (Top Bar)-->
                    <div class="container-xxl d-flex flex-grow-1 flex-stack">
                        <!--begin::Header Logo-->
                        <div class="d-flex align-items-center me-5">
                            <!--begin::Sidebar toggle (mobile)-->
                            <div class="d-lg-none btn btn-icon btn-active-color-primary w-30px h-30px ms-n2 me-3" id="kt_app_sidebar_toggle">
                                <i class="ki-duotone ki-abstract-14 fs-2"><span class="path1"></span><span class="path2"></span></i>
                            </div>
                            <!--end::Sidebar toggle-->
                            <!--begin::Header menu toggle (mobile)-->
                            <div class="d-lg-none btn btn-icon btn-active-color-primary w-30px h-30px me-3" id="kt_header_menu_toggle">
                                <i class="ki-duotone ki-text-align-left fs-2"><span class="path1"></span><span class="path2"></span><span class="path3"></span><span class="path4"></span></i>
                            </div>
                            <!--end::Header menu toggle-->
                            <a href="{{ route('dashboard') }}">
                                <img alt="Logo" src="{{ asset('assets/media/logos/' . $siteLogo) }}" class="theme-light-show h-20px h-lg-30px" />
                                <img alt="Logo" src="{{ asset('assets/media/logos/' . $siteLogo) }}" class="theme-dark-show h-20px h-lg-30px" />
                            </a>
                        </div>
                        <!--end::Header Logo-->
                        <!--begin::Topbar-->
                        @include('backend.layout.navbar')
                        <!--end::Topbar-->
                    </div>
                    <!--end::Container-->
                    <!--begin::Separator-->
                    <div class="separator"></div>
                    <!--end::Separator-->
                    <!--begin::Container (Menu Bar)-->
                    <div class="header-menu-container container-xxl d-flex flex-stack h-lg-75px w-100" id="kt_header_nav">
                        <!--begin::Menu wrapper-->
                        @include('backend.layout.menu')
                        <!--end::Menu wrapper-->
                    </div>
                    <!--end::Container-->
                </div>
                <!--end::Header-->

                <!--begin::Container-->
                <div id="kt_content_container" class="d-flex flex-column-fluid align-items-start container-xxl">
                    <!--begin::Sidebar (Custom for Demo 11)-->
                    @include('backend.layout.sidebar')
                    <!--end::Sidebar-->
                    <!--begin::Content-->
                    <div class="content flex-row-fluid" id="kt_content">
                        @yield('content')
                    </div>
                    <!--end::Content-->
                </div>
                <!--end::Container-->

                <!--begin::Footer-->
                @include('backend.layout.footer')
                <!--end::Footer-->
            </div>
            <!--end::Wrapper-->
        </div>
        <!--end::Page-->
    </div>
    <!--end::Root-->
    <!--end::Main-->

    <!--begin::Sidebar Overlay (mobile)-->
    <div class="sidebar-overlay" id="kt_sidebar_overlay"></div>

    <!--begin::Javascript-->
    <script>
        var hostUrl = "{{ asset('assets/') }}";
    </script>
    <script src="{{ asset('assets/plugins/global/plugins.bundle.js') }}"></script>
    <script src="{{ asset('assets/js/scripts.bundle.js') }}"></script>
    <script src="{{ asset('assets/plugins/custom/fullcalendar/fullcalendar.bundle.js') }}"></script>
    <script src="{{ asset('assets/plugins/custom/datatables/datatables.bundle.js') }}"></script>
    <script src="{{ asset('assets/js/widgets.bundle.js') }}"></script>
    <script src="{{ asset('assets/js/custom/widgets.js') }}"></script>
    <script src="{{ asset('assets/js/custom/apps/chat/chat.js') }}"></script>
    <script src="{{ asset('assets/js/custom/utilities/modals/create-campaign.js') }}"></script>
    <script src="{{ asset('assets/js/custom/utilities/modals/users-search.js') }}"></script>
    <script>
        document.addEventListener('DOMContentLoaded', function() {
            // --- Sidebar Toggle ---
            const sidebar = document.getElementById('kt_app_sidebar');
            const overlay = document.getElementById('kt_sidebar_overlay');
            const toggleBtns = document.querySelectorAll('#kt_app_sidebar_toggle, #kt_sidebar_toggle_desktop');

            toggleBtns.forEach(btn => {
                btn.addEventListener('click', function() {
                    sidebar.classList.toggle('active');
                    overlay.classList.toggle('active');
                });
            });
            if (overlay) {
                overlay.addEventListener('click', function() {
                    sidebar.classList.remove('active');
                    overlay.classList.remove('active');
                });
            }

            // --- Global Toastr ---
            toastr.options = {
                "closeButton": true,
                "progressBar": true,
                "positionClass": "toastr-top-right",
                "timeOut": "5000"
            };
            @if(session('success')) toastr.success("{{ session('success') }}"); @endif
            @if(session('error')) toastr.error("{{ session('error') }}"); @endif
            @if(session('warning')) toastr.warning("{{ session('warning') }}"); @endif
            @if(session('info')) toastr.info("{{ session('info') }}"); @endif

            // --- Quick Search ---
            const searchPages = [
                { title: 'Dashboard', url: "{{ route('dashboard') }}", icon: 'ki-element-11', desc: 'Main dashboard overview' },
                { title: 'Settings', url: "{{ route('settings.index') }}", icon: 'ki-setting-2', desc: 'Application configuration' }
            ];
            const searchInput = document.querySelector('[data-kt-search-element="input"]');
            const resultsEl = document.querySelector('[data-kt-search-element="results"]');
            const mainEl = document.querySelector('[data-kt-search-element="main"]');
            const emptyEl = document.querySelector('[data-kt-search-element="empty"]');
            const resultsContainer = document.getElementById('kt_header_search_results');
            if(searchInput) {
                searchInput.addEventListener('input', function(e) {
                    const query = e.target.value.toLowerCase();
                    if(query.length > 1) {
                        mainEl.classList.add('d-none');
                        const filtered = searchPages.filter(p => p.title.toLowerCase().includes(query) || p.desc.toLowerCase().includes(query));
                        if(filtered.length > 0) {
                            emptyEl.classList.add('d-none');
                            resultsEl.classList.remove('d-none');
                            let html = '';
                            filtered.forEach(p => {
                                html += `<a href="${p.url}" class="d-flex text-gray-900 text-hover-primary align-items-center mb-5"><div class="symbol symbol-40px me-4"><span class="symbol-label bg-light"><i class="ki-duotone ${p.icon} fs-2 text-primary"><span class="path1"></span><span class="path2"></span><span class="path3"></span><span class="path4"></span></i></span></div><div class="d-flex flex-column"><span class="fs-6 fw-bold">${p.title}</span><span class="fs-7 fw-semibold text-muted">${p.desc}</span></div></a>`;
                            });
                            resultsContainer.innerHTML = html;
                        } else {
                            resultsEl.classList.add('d-none');
                            emptyEl.classList.remove('d-none');
                        }
                    } else {
                        mainEl.classList.remove('d-none');
                        resultsEl.classList.add('d-none');
                        emptyEl.classList.add('d-none');
                    }
                });
            }

            // --- Force Logout Listener ---
            @auth
            const userId = "{{ auth()->id() }}";
            const waitForEchoLogout = setInterval(() => {
                if (window.Echo) {
                    clearInterval(waitForEchoLogout);
                    window.Echo.private(`App.Models.User.${userId}`)
                        .listen('ForceLogoutNotification', (e) => {
                            Swal.fire({
                                title: 'Keamanan Akun', text: e.message, icon: 'warning',
                                allowOutsideClick: false, allowEscapeKey: false,
                                confirmButtonText: 'OK, Logout', confirmButtonColor: '#d33'
                            }).then(() => { window.location.href = "{{ route('login') }}"; });
                        });
                }
            }, 500);
            @endauth
        });
    </script>
    <!--end::Javascript-->
    @stack('scripts')
</body>
<!--end::Body-->
</html>
