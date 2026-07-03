@extends('backend.layout.app')
@section('title', 'Settings')
@section('content')

    <div class="mt-5 mb-10">

            {{-- Flash Messages --}}
            @if(session('success'))
                <div class="alert alert-success d-flex align-items-center p-5 mb-6">
                    <i class="ki-outline ki-shield-tick fs-2hx text-success me-4"></i>
                    <div class="d-flex flex-column">
                        <h4 class="mb-1 text-success">Berhasil!</h4>
                        <span>{{ session('success') }}</span>
                    </div>
                    <button type="button" class="btn-close ms-auto" data-bs-dismiss="alert"></button>
                </div>
            @endif

            {{-- Page Header --}}
            <div class="d-flex flex-column flex-lg-row mb-8">
                <div class="flex-lg-row-fluid">
                    <h1 class="fw-bold fs-2x mb-2">Settings</h1>
                    <div class="text-muted fw-semibold fs-6">Manage your application configuration</div>
                </div>
            </div>

            <form method="POST" action="{{ route('settings.update') }}" enctype="multipart/form-data" id="settingsForm">
                @csrf

                {{-- Tabs Navigation --}}
                <ul class="nav nav-custom nav-tabs nav-line-tabs nav-line-tabs-2x border-0 fs-5 fw-semibold mb-8" role="tablist">
                    <li class="nav-item" role="presentation">
                        <a class="nav-link text-active-primary pb-4 active" data-bs-toggle="tab" href="#tab_general" role="tab">
                            <i class="ki-outline ki-setting-2 fs-4 me-2"></i>General
                        </a>
                    </li>
                    <li class="nav-item" role="presentation">
                        <a class="nav-link text-active-primary pb-4" data-bs-toggle="tab" href="#tab_social" role="tab">
                            <i class="ki-outline ki-fingerprint-scanning fs-4 me-2"></i>Social Login
                        </a>
                    </li>
                </ul>

                {{-- Tab Content --}}
                <div class="tab-content">

                    {{-- ==================== TAB: GENERAL ==================== --}}
                    <div class="tab-pane fade show active" id="tab_general" role="tabpanel">
                        <div class="row g-6">

                            {{-- Card: App Identity --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <i class="ki-outline ki-abstract-26 fs-3 me-2 text-primary"></i>App Identity
                                        </h3>
                                    </div>
                                    <div class="card-body pt-2">
                                        <div class="fv-row mb-7">
                                            <label class="fs-6 fw-semibold mb-2">Application Name</label>
                                            <input type="text" class="form-control form-control-solid" name="site_name"
                                                value="{{ $settings['site_name'] ?? 'StarterTemp' }}" placeholder="Your App Name" />
                                        </div>

                                        <div class="fv-row mb-3">
                                            <label class="fs-6 fw-semibold mb-2">Application Logo</label>
                                            <div class="d-flex align-items-center mb-4">
                                                <div class="symbol symbol-100px symbol-fixed me-5 border rounded p-2 bg-light">
                                                    <img src="{{ asset('assets/media/logos/' . ($settings['site_logo'] ?? 'base-logo.png')) }}"
                                                        alt="Current Logo" id="logoPreview" class="mw-100" />
                                                </div>
                                                <div class="flex-grow-1">
                                                    <input type="file" class="form-control form-control-solid" name="site_logo"
                                                        accept=".png,.jpg,.jpeg,.svg,.webp" onchange="previewLogo(this)" />
                                                    <div class="form-text text-muted mt-2">Accepted: PNG, JPG, SVG, WebP. Max 2MB.</div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {{-- Card: Appearance --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <i class="ki-outline ki-design-1 fs-3 me-2 text-info"></i>Appearance
                                        </h3>
                                    </div>
                                    <div class="card-body pt-2">
                                        <div class="fv-row mb-7">
                                            <label class="fs-6 fw-semibold mb-2">Global Font Family</label>
                                            <select class="form-select form-select-solid" name="site_font" id="fontSelector">
                                                @foreach($fonts as $fontName => $fontUrl)
                                                    <option value="{{ $fontName }}"
                                                        {{ ($settings['site_font'] ?? 'Plus Jakarta Sans') === $fontName ? 'selected' : '' }}>
                                                        {{ $fontName }}
                                                    </option>
                                                @endforeach
                                            </select>
                                            <div class="form-text text-muted mt-2">This font will apply globally across the entire application.</div>
                                        </div>

                                        <div class="fv-row mb-7">
                                            <div class="d-flex flex-stack">
                                                <div class="d-flex flex-column">
                                                    <span class="fs-6 fw-bold text-dark">Maintenance Mode</span>
                                                    <span class="fs-7 fw-semibold text-muted">Activate maintenance mode to restrict access (Superadmin bypasses this).</span>
                                                </div>
                                                <div class="form-check form-switch form-check-custom form-check-solid">
                                                    <input class="form-check-input h-25px w-45px" type="checkbox" name="maintenance_mode" {{ ($settings['maintenance_mode'] ?? '0') === '1' ? 'checked' : '' }} />
                                                </div>
                                            </div>
                                        </div>

                                        <div class="fv-row">
                                            <label class="fs-6 fw-semibold mb-3">Font Preview</label>
                                            <div class="border rounded p-5 bg-light" id="fontPreviewBox">
                                                <div class="fs-2 fw-bold mb-2" id="fontPreviewTitle">The quick brown fox jumps over the lazy dog</div>
                                                <div class="fs-6 text-muted" id="fontPreviewBody">ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                        </div>
                    </div>

                    {{-- ==================== TAB: SOCIAL LOGIN ==================== --}}
                    <div class="tab-pane fade" id="tab_social" role="tabpanel">

                        <div class="notice d-flex bg-light-primary rounded border-primary border border-dashed p-6 mb-8">
                            <i class="ki-outline ki-information-5 fs-2tx text-primary me-4"></i>
                            <div class="d-flex flex-stack flex-grow-1">
                                <div class="fw-semibold">
                                    <h4 class="text-gray-900 fw-bold">Social Login Configuration</h4>
                                    <div class="fs-6 text-gray-700">
                                        Enable social login providers and configure their OAuth credentials.
                                        Users will be able to login with enabled providers on the login page.
                                        <br><strong>Callback URL format:</strong>
                                        <code>{{ url('/admin/auth/{provider}/callback') }}</code>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="row g-6">

                            {{-- Google --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <svg class="me-2" width="22" height="22" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                                                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                                                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                                                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                                            </svg>
                                            Google
                                        </h3>
                                        <div class="card-toolbar">
                                            <div class="form-check form-switch form-check-custom form-check-solid">
                                                <input class="form-check-input h-25px w-45px" type="checkbox" name="social_google_enabled"
                                                    {{ ($settings['social_google_enabled'] ?? '0') === '1' ? 'checked' : '' }}
                                                    onchange="toggleProviderFields(this, 'google')" />
                                            </div>
                                        </div>
                                    </div>
                                    <div class="card-body pt-2" id="google_fields">
                                        <div class="fv-row mb-5">
                                            <label class="fs-6 fw-semibold mb-2">Client ID</label>
                                            <input type="text" class="form-control form-control-solid" name="social_google_client_id"
                                                value="{{ $settings['social_google_client_id'] ?? '' }}" placeholder="Google Client ID" />
                                        </div>
                                        <div class="fv-row mb-3">
                                            <label class="fs-6 fw-semibold mb-2">Client Secret</label>
                                            <input type="password" class="form-control form-control-solid" name="social_google_client_secret"
                                                placeholder="Leave empty to keep existing" />
                                        </div>
                                        <div class="fv-row">
                                            <label class="fs-7 fw-semibold text-muted">Callback URL</label>
                                            <div class="form-control form-control-solid bg-light-primary text-primary fs-7">{{ url('/admin/auth/google/callback') }}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {{-- Facebook --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <svg class="me-2" width="22" height="22" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z" fill="#1877F2"/>
                                            </svg>
                                            Facebook
                                        </h3>
                                        <div class="card-toolbar">
                                            <div class="form-check form-switch form-check-custom form-check-solid">
                                                <input class="form-check-input h-25px w-45px" type="checkbox" name="social_facebook_enabled"
                                                    {{ ($settings['social_facebook_enabled'] ?? '0') === '1' ? 'checked' : '' }}
                                                    onchange="toggleProviderFields(this, 'facebook')" />
                                            </div>
                                        </div>
                                    </div>
                                    <div class="card-body pt-2" id="facebook_fields">
                                        <div class="fv-row mb-5">
                                            <label class="fs-6 fw-semibold mb-2">Client ID</label>
                                            <input type="text" class="form-control form-control-solid" name="social_facebook_client_id"
                                                value="{{ $settings['social_facebook_client_id'] ?? '' }}" placeholder="Facebook App ID" />
                                        </div>
                                        <div class="fv-row mb-3">
                                            <label class="fs-6 fw-semibold mb-2">Client Secret</label>
                                            <input type="password" class="form-control form-control-solid" name="social_facebook_client_secret"
                                                placeholder="Leave empty to keep existing" />
                                        </div>
                                        <div class="fv-row">
                                            <label class="fs-7 fw-semibold text-muted">Callback URL</label>
                                            <div class="form-control form-control-solid bg-light-primary text-primary fs-7">{{ url('/admin/auth/facebook/callback') }}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {{-- GitHub --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <svg class="me-2" width="22" height="22" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" fill="#333"/>
                                            </svg>
                                            GitHub
                                        </h3>
                                        <div class="card-toolbar">
                                            <div class="form-check form-switch form-check-custom form-check-solid">
                                                <input class="form-check-input h-25px w-45px" type="checkbox" name="social_github_enabled"
                                                    {{ ($settings['social_github_enabled'] ?? '0') === '1' ? 'checked' : '' }}
                                                    onchange="toggleProviderFields(this, 'github')" />
                                            </div>
                                        </div>
                                    </div>
                                    <div class="card-body pt-2" id="github_fields">
                                        <div class="fv-row mb-5">
                                            <label class="fs-6 fw-semibold mb-2">Client ID</label>
                                            <input type="text" class="form-control form-control-solid" name="social_github_client_id"
                                                value="{{ $settings['social_github_client_id'] ?? '' }}" placeholder="GitHub Client ID" />
                                        </div>
                                        <div class="fv-row mb-3">
                                            <label class="fs-6 fw-semibold mb-2">Client Secret</label>
                                            <input type="password" class="form-control form-control-solid" name="social_github_client_secret"
                                                placeholder="Leave empty to keep existing" />
                                        </div>
                                        <div class="fv-row">
                                            <label class="fs-7 fw-semibold text-muted">Callback URL</label>
                                            <div class="form-control form-control-solid bg-light-primary text-primary fs-7">{{ url('/admin/auth/github/callback') }}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {{-- LinkedIn --}}
                            <div class="col-xl-6">
                                <div class="card shadow-sm h-100">
                                    <div class="card-header border-0 pt-6">
                                        <h3 class="card-title fw-bold fs-4">
                                            <svg class="me-2" width="22" height="22" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                                <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" fill="#0A66C2"/>
                                            </svg>
                                            LinkedIn
                                        </h3>
                                        <div class="card-toolbar">
                                            <div class="form-check form-switch form-check-custom form-check-solid">
                                                <input class="form-check-input h-25px w-45px" type="checkbox" name="social_linkedin_enabled"
                                                    {{ ($settings['social_linkedin_enabled'] ?? '0') === '1' ? 'checked' : '' }}
                                                    onchange="toggleProviderFields(this, 'linkedin')" />
                                            </div>
                                        </div>
                                    </div>
                                    <div class="card-body pt-2" id="linkedin_fields">
                                        <div class="fv-row mb-5">
                                            <label class="fs-6 fw-semibold mb-2">Client ID</label>
                                            <input type="text" class="form-control form-control-solid" name="social_linkedin_client_id"
                                                value="{{ $settings['social_linkedin_client_id'] ?? '' }}" placeholder="LinkedIn Client ID" />
                                        </div>
                                        <div class="fv-row mb-3">
                                            <label class="fs-6 fw-semibold mb-2">Client Secret</label>
                                            <input type="password" class="form-control form-control-solid" name="social_linkedin_client_secret"
                                                placeholder="Leave empty to keep existing" />
                                        </div>
                                        <div class="fv-row">
                                            <label class="fs-7 fw-semibold text-muted">Callback URL</label>
                                            <div class="form-control form-control-solid bg-light-primary text-primary fs-7">{{ url('/admin/auth/linkedin-openid/callback') }}</div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                        </div>
                    </div>

                </div>{{-- end tab-content --}}

                {{-- Save Button (Sticky Bottom) --}}
                <div class="d-flex justify-content-end mt-8 mb-10">
                    <button type="submit" class="btn btn-primary px-8 py-3 fw-bold fs-5" id="saveBtn">
                        <i class="ki-outline ki-check fs-3 me-1"></i> Save Changes
                    </button>
                </div>
            </form>
        </div>

@endsection

@push('scripts')
<script>
    // Logo Preview
    function previewLogo(input) {
        if (input.files && input.files[0]) {
            const reader = new FileReader();
            reader.onload = function(e) {
                document.getElementById('logoPreview').src = e.target.result;
            };
            reader.readAsDataURL(input.files[0]);
        }
    }

    // Toggle provider credential fields opacity
    function toggleProviderFields(checkbox, provider) {
        const fields = document.getElementById(provider + '_fields');
        if (fields) {
            fields.style.opacity = checkbox.checked ? '1' : '0.4';
            fields.style.pointerEvents = checkbox.checked ? 'auto' : 'none';
        }
    }

    // Initialize field states on page load
    document.addEventListener('DOMContentLoaded', function() {
        ['google', 'facebook', 'github', 'linkedin'].forEach(provider => {
            const checkbox = document.querySelector(`[name="social_${provider}_enabled"]`);
            if (checkbox) {
                toggleProviderFields(checkbox, provider);
            }
        });
    });

    // Font Preview
    const fontSelector = document.getElementById('fontSelector');
    const fontUrls = @json($fonts);

    function updateFontPreview() {
        const selectedFont = fontSelector.value;
        const googleUrl = fontUrls[selectedFont];

        // Load font dynamically
        const linkId = 'preview-font-link';
        let link = document.getElementById(linkId);
        if (!link) {
            link = document.createElement('link');
            link.id = linkId;
            link.rel = 'stylesheet';
            document.head.appendChild(link);
        }
        link.href = `https://fonts.googleapis.com/css2?family=${googleUrl}&display=swap`;

        const previewBox = document.getElementById('fontPreviewBox');
        previewBox.style.fontFamily = `'${selectedFont}', sans-serif`;
    }

    fontSelector.addEventListener('change', updateFontPreview);
    updateFontPreview(); // Initial preview
</script>
@endpush
