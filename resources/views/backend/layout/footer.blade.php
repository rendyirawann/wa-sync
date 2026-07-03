<!--begin::Footer-->
<div class="footer py-4 d-flex flex-lg-column" id="kt_footer">
    <div class="container-xxl d-flex flex-column flex-md-row flex-stack">
        <!--begin::Copyright-->
        <div class="text-gray-900 order-2 order-md-1">
            <span class="text-muted fw-semibold me-1">{{ date('Y') }}&copy;</span>
            <a href="#" target="_blank" class="text-gray-800 text-hover-primary">{{ $appSettings['site_name'] ?? 'StarterTemp' }}</a>
        </div>
        <!--end::Copyright-->
        <!--begin::Menu-->
        <ul class="menu menu-gray-600 menu-hover-primary fw-semibold order-1">
            <li class="menu-item">
                <a href="{{ route('dashboard') }}" class="menu-link px-2">Dashboard</a>
            </li>
            <li class="menu-item">
                <a href="{{ route('settings.index') }}" class="menu-link px-2">Settings</a>
            </li>
        </ul>
        <!--end::Menu-->
    </div>
</div>
<!--end::Footer-->