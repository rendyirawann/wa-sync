{{-- Sidebar (Custom addition for Demo 11 layout) --}}
<div id="kt_app_sidebar" class="d-flex flex-column">

    <!--begin::Sidebar Header-->
    <div class="d-flex flex-column align-items-center px-6 pt-8 pb-5">
        @php $siteLogo = $appSettings['site_logo'] ?? 'base-logo.png'; @endphp
        <a href="{{ route('dashboard') }}" class="mb-4">
            <img alt="Logo" src="{{ asset('assets/media/logos/' . $siteLogo) }}" class="h-40px" />
        </a>
        <h5 class="fw-bold text-gray-800 mb-0 fs-6">{{ $appSettings['site_name'] ?? 'StarterTemp' }}</h5>
        <span class="text-muted fs-8">Navigation</span>
    </div>
    <!--end::Sidebar Header-->

    <div class="separator mx-6 mb-3"></div>

    <!--begin::Sidebar Menu-->
    <div class="px-4 flex-column-fluid">
        <div class="menu menu-column menu-rounded menu-sub-indention menu-active-bg fw-semibold fs-6" data-kt-menu="true">

            <!--begin::Dashboard-->
            <div class="menu-item">
                <a class="menu-link {{ request()->routeIs('dashboard') ? 'active' : '' }}" href="{{ route('dashboard') }}">
                    <span class="menu-icon">
                        <i class="ki-duotone ki-element-11 fs-3"><span class="path1"></span><span class="path2"></span><span class="path3"></span><span class="path4"></span></i>
                    </span>
                    <span class="menu-title">Dashboard</span>
                </a>
            </div>
            <!--end::Dashboard-->

            <!--begin::Section: Pages-->
            <div class="menu-item pt-5">
                <div class="menu-content">
                    <span class="menu-heading fw-bold text-uppercase fs-7">Pages</span>
                </div>
            </div>

            <!--begin::User Management-->
            @role('Superadmin|superadmin')
            <div data-kt-menu-trigger="click" class="menu-item menu-accordion {{ request()->is('admin/users*') || request()->is('admin/roles*') ? 'here show' : '' }}">
                <span class="menu-link">
                    <span class="menu-icon">
                        <i class="ki-duotone ki-people fs-3"><span class="path1"></span><span class="path2"></span><span class="path3"></span><span class="path4"></span><span class="path5"></span></i>
                    </span>
                    <span class="menu-title">User Management</span>
                    <span class="menu-arrow"></span>
                </span>
                <div class="menu-sub menu-sub-accordion {{ request()->is('admin/users*') || request()->is('admin/roles*') ? 'show' : '' }}">
                    <div class="menu-item">
                        <a class="menu-link {{ request()->is('admin/users*') ? 'active' : '' }}" href="{{ route('users.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Users</span>
                        </a>
                    </div>
                    <div class="menu-item">
                        <a class="menu-link {{ request()->is('admin/roles*') ? 'active' : '' }}" href="{{ route('roles.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Roles & Permissions</span>
                        </a>
                    </div>
                </div>
            </div>
            @endrole
            <!--end::User Management-->

            <!--begin::My Account-->
            <div data-kt-menu-trigger="click" class="menu-item menu-accordion {{ request()->is('admin/my-*') ? 'here show' : '' }}">
                <span class="menu-link">
                    <span class="menu-icon">
                        <i class="ki-duotone ki-profile-circle fs-3"><span class="path1"></span><span class="path2"></span><span class="path3"></span></i>
                    </span>
                    <span class="menu-title">My Account</span>
                    <span class="menu-arrow"></span>
                </span>
                <div class="menu-sub menu-sub-accordion {{ request()->is('admin/my-*') ? 'show' : '' }}">
                    <div class="menu-item">
                        <a class="menu-link {{ request()->routeIs('account.index') ? 'active' : '' }}" href="{{ route('account.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Overview</span>
                        </a>
                    </div>
                    <div class="menu-item">
                        <a class="menu-link {{ request()->is('admin/my-security*') ? 'active' : '' }}" href="{{ route('my-security.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Security</span>
                        </a>
                    </div>
                    <div class="menu-item">
                        <a class="menu-link {{ request()->is('admin/my-activity*') ? 'active' : '' }}" href="{{ route('my-activity.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Activity</span>
                        </a>
                    </div>
                    <div class="menu-item">
                        <a class="menu-link {{ request()->is('admin/mmy-login-session*') ? 'active' : '' }}" href="{{ route('my-login-session.index') }}">
                            <span class="menu-bullet"><span class="bullet bullet-dot"></span></span>
                            <span class="menu-title">Login Sessions</span>
                        </a>
                    </div>
                </div>
            </div>
            <!--end::My Account-->

            <!--begin::Section: System-->
            @role('Superadmin|superadmin')
            <div class="menu-item pt-5">
                <div class="menu-content">
                    <span class="menu-heading fw-bold text-uppercase fs-7">System</span>
                </div>
            </div>

            <!--begin::Settings-->
            <div class="menu-item">
                <a class="menu-link {{ request()->routeIs('settings.*') ? 'active' : '' }}" href="{{ route('settings.index') }}">
                    <span class="menu-icon">
                        <i class="ki-duotone ki-setting-2 fs-3"><span class="path1"></span><span class="path2"></span></i>
                    </span>
                    <span class="menu-title">Settings</span>
                </a>
            </div>
            <!--end::Settings-->

            <!--begin::Activity Log-->
            <div class="menu-item">
                <a class="menu-link {{ request()->is('admin/log-activity*') ? 'active' : '' }}" href="{{ url('admin/log-activity') }}">
                    <span class="menu-icon">
                        <i class="ki-duotone ki-notepad fs-3"><span class="path1"></span><span class="path2"></span><span class="path3"></span><span class="path4"></span><span class="path5"></span></i>
                    </span>
                    <span class="menu-title">Activity Log</span>
                </a>
            </div>
            <!--end::Activity Log-->
            @endrole

        </div>
    </div>
    <!--end::Sidebar Menu-->

    <!--begin::Sidebar Footer-->
    <div class="px-6 py-5 mt-auto">
        <div class="separator mb-4"></div>
        <div class="d-flex align-items-center">
            @auth
            <div class="symbol symbol-35px me-3">
                <img alt="Avatar" src="{{ asset('assets/media/avatars/' . (Auth::user()->avatar ?? 'default.png')) }}" />
            </div>
            <div class="d-flex flex-column flex-grow-1">
                <span class="fw-bold fs-7 text-gray-800">{{ Auth::user()->name }}</span>
                <span class="text-muted fs-8">{{ Auth::user()->roles->first()->name ?? 'User' }}</span>
            </div>
            @endauth
        </div>
    </div>
    <!--end::Sidebar Footer-->
</div>
