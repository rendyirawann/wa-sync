@extends('auth.app')
@section('title', 'Login')

@push('stylesheets')
    <style>
        .floating-manual-btn {
            position: fixed;
            right: 30px;
            bottom: 30px;
            z-index: 1050;
            width: 60px;
            height: 60px;
            background-color: #0095E8;
            color: white;
            border-radius: 50px;
            cursor: pointer;
            box-shadow: 0 10px 30px rgba(0, 149, 232, 0.4);
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
            overflow: hidden;
            white-space: nowrap;
        }

        .floating-manual-btn i {
            font-size: 1.8rem;
            transition: transform 0.3s ease;
        }

        .floating-manual-btn .btn-text {
            opacity: 0;
            width: 0;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            transition: all 0.3s ease;
            margin-left: 0;
        }

        .floating-manual-btn:hover {
            width: 180px;
            background-color: #007bbd;
            border-radius: 30px;
        }

        .floating-manual-btn:hover i {
            transform: rotate(-10deg) scale(0.9);
        }

        .floating-manual-btn:hover .btn-text {
            opacity: 1;
            width: auto;
            margin-left: 10px;
        }

        /* Pulse Animation */
        .pulse-effect {
            position: absolute;
            width: 100%;
            height: 100%;
            border-radius: 50%;
            background-color: rgba(0, 149, 232, 0.5);
            animation: pulse-animation 2s infinite;
            z-index: -1;
        }

        @keyframes pulse-animation {
            0% { transform: scale(1); opacity: 0.6; }
            100% { transform: scale(1.6); opacity: 0; }
        }

        /* Viewer Styles */
        .manual-page-img {
            max-width: 100%;
            height: auto;
            border-radius: 8px;
            box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
            display: none;
        }

        .manual-page-img.active {
            display: block;
            animation: fadeIn 0.4s ease-in-out;
        }

        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }

        @keyframes slideInRight {
            from { transform: translateX(50px); opacity: 0; }
            to { transform: translateX(0); opacity: 1; }
        }

        .manual-pdf-container {
            width: 100%;
            height: auto;
            max-height: calc(100vh - 180px);
            display: none;
            justify-content: center;
            align-items: center;
            overflow: auto;
        }

        .manual-pdf-container.active {
            display: flex;
            animation: slideInRight 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
        }

        #pdf-render {
            max-width: 100%;
            height: auto;
            border-radius: 8px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.15);
        }

        .pdf-loader {
            display: none;
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }
    </style>
@endpush

@section('content')
    <div class="d-flex flex-column-fluid flex-lg-row-auto justify-content-center justify-content-lg-start p-12">

        <div class="bg-body d-flex flex-column flex-center rounded-4 w-md-600px p-10 shadow-lg">

            <div class="d-flex flex-center flex-column align-items-stretch h-lg-100 w-md-400px">

                <div class="d-flex flex-center flex-column flex-column-fluid mb-2">
                    <img alt="Logo" class="theme-light-show h-40px h-lg-150px"
                        src="{{ asset('assets/media/logos/base-logo.png') }}" />
                    <img alt="Logo" class="theme-dark-show h-40px h-lg-150px"
                        src="{{ asset('assets/media/logos/base-logo.png') }}" />
                </div>

                <div class="d-flex flex-center flex-column flex-column-fluid pb-15 pb-lg-20 my-12">

                    <form class="form w-100" id="kt_sign_in_form" method="POST" action="{{ route('login') }}">
                        @csrf

                        <div class="fv-row mb-8">
                            <input type="text" placeholder="Email / No WA / Nama User" name="email" autocomplete="off"
                                class="form-control bg-transparent" />
                        </div>

                        <div class="fv-row mb-3 position-relative" data-kt-password-meter="true">
                            <input type="password" placeholder="Password" name="password" autocomplete="off"
                                class="form-control bg-transparent" id="passwordInput" />

                            <span class="btn btn-sm btn-icon position-absolute translate-middle top-50 end-0 me-n2"
                                id="togglePassword">
                                <i class="ki-outline ki-eye-slash fs-2" id="toggleIcon"></i>
                            </span>
                        </div>

                        <div class="d-flex flex-stack flex-wrap gap-3 fs-base fw-semibold mb-8">
                            <div></div>
                            <a href="{{ route('password.request') }}" class="link-primary">Lupa Password ?</a>
                        </div>

                        <div class="d-grid mb-10">
                            <button type="submit" id="kt_sign_in_submit" class="btn btn-primary">
                                <span class="indicator-label">Masuk</span>
                                <span class="indicator-progress">Harap tunggu...
                                    <span class="spinner-border spinner-border-sm align-middle ms-2"></span></span>
                            </button>
                        </div>

                        <!-- <div class="text-gray-500 text-center fw-semibold fs-6">Belum punya akun?
                            <a href="{{ route('register') }}" class="link-primary">Daftar</a>
                        </div> -->
                    </form>

                    {{-- Social Login Section --}}
                    @php
                        $socialProviders = [
                            'google' => [
                                'enabled' => ($appSettings['social_google_enabled'] ?? '0') === '1',
                                'label' => 'Google',
                                'driver' => 'google',
                                'icon' => '<svg width="20" height="20" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>',
                                'color' => '#fff',
                                'border' => '#dadce0',
                                'text' => '#3c4043',
                            ],
                            'facebook' => [
                                'enabled' => ($appSettings['social_facebook_enabled'] ?? '0') === '1',
                                'label' => 'Facebook',
                                'driver' => 'facebook',
                                'icon' => '<svg width="20" height="20" viewBox="0 0 24 24"><path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z" fill="#1877F2"/></svg>',
                                'color' => '#1877F2',
                                'border' => '#1877F2',
                                'text' => '#fff',
                            ],
                            'github' => [
                                'enabled' => ($appSettings['social_github_enabled'] ?? '0') === '1',
                                'label' => 'GitHub',
                                'driver' => 'github',
                                'icon' => '<svg width="20" height="20" viewBox="0 0 24 24"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" fill="#333"/></svg>',
                                'color' => '#24292f',
                                'border' => '#24292f',
                                'text' => '#fff',
                            ],
                            'linkedin' => [
                                'enabled' => ($appSettings['social_linkedin_enabled'] ?? '0') === '1',
                                'label' => 'LinkedIn',
                                'driver' => 'linkedin-openid',
                                'icon' => '<svg width="20" height="20" viewBox="0 0 24 24"><path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" fill="#0A66C2"/></svg>',
                                'color' => '#0A66C2',
                                'border' => '#0A66C2',
                                'text' => '#fff',
                            ],
                        ];
                        $hasAnySocial = collect($socialProviders)->contains('enabled', true);
                    @endphp

                    @if($hasAnySocial)
                        <div class="separator separator-content my-8">
                            <span class="text-gray-500 fw-semibold fs-7">Atau masuk dengan</span>
                        </div>

                        <div class="d-flex flex-wrap justify-content-center gap-3">
                            @foreach($socialProviders as $key => $provider)
                                @if($provider['enabled'])
                                    <a href="{{ route('social.redirect', $provider['driver']) }}"
                                        class="btn btn-flex btn-outline btn-text-gray-700 btn-active-color-primary bg-state-light flex-center"
                                        style="border-color: {{ $provider['border'] }}; min-width: 140px;">
                                        {!! $provider['icon'] !!}
                                        <span class="ms-2 fs-7 fw-bold">{{ $provider['label'] }}</span>
                                    </a>
                                @endif
                            @endforeach
                        </div>
                    @endif

                </div>
            </div>
        </div>
    </div>

    <!-- Floating Button Version 2: FAB (Floating Action Button) -->
    <div class="floating-manual-btn shadow-lg" data-bs-toggle="modal" data-bs-target="#modal_manual_book">
        <div class="pulse-effect"></div>
        <i class="ki-outline ki-book-open"></i>
        <span class="btn-text">Manual Book</span>
    </div>

    <!-- Modal Selection -->
    <div class="modal fade" id="modal_manual_book" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered mw-650px">
            <div class="modal-content">
                <div class="modal-header">
                    <h2 class="fw-bold">Manual Book</h2>
                    <div class="btn btn-icon btn-sm btn-active-icon-primary" data-bs-dismiss="modal">
                        <i class="ki-outline ki-cross fs-1"></i>
                    </div>
                </div>
                <div class="modal-body scroll-y mx-5 mx-xl-15 my-7">
                    <div class="text-center mb-13">
                        <h1 class="mb-3">Pilih Panduan</h1>
                        <div class="text-muted fw-semibold fs-5">Silakan pilih kategori panduan yang ingin Anda pelajari</div>
                    </div>

                    <div class="row g-10">
                        <div class="col-md-12">
                            <div class="mb-5">
                                <div class="d-flex flex-stack cursor-pointer p-8 rounded-3 border border-dashed border-gray-300 bg-light-primary bg-hover-light"
                                    onclick="openViewer()">
                                    <div class="d-flex align-items-center me-2">
                                        <div class="symbol symbol-50px symbol-circle me-3">
                                            <div class="symbol-label bg-primary">
                                                <i class="ki-outline ki-document text-white fs-2"></i>
                                            </div>
                                        </div>
                                        <div class="py-1">
                                            <a href="#" class="fs-4 fw-bold text-gray-800 text-hover-primary mb-1">Panduan POS & Kiosk</a>
                                            <div class="fs-6 fw-semibold text-gray-400">Total 3 Halaman</div>
                                        </div>
                                    </div>
                                    <button class="btn btn-sm btn-primary">Lihat</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Modal Viewer (Virtual PDF) -->
    <div class="modal fade" id="modal_manual_viewer" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-fullscreen">
            <div class="modal-content">
                <div class="modal-header py-3">
                    <h3 class="fw-bold m-0">Manual Book Viewer</h3>
                    <div class="btn btn-icon btn-sm btn-active-icon-primary" data-bs-dismiss="modal">
                        <i class="ki-outline ki-cross fs-1"></i>
                    </div>
                </div>
                <div class="modal-body p-0 bg-gray-200">
                    <div class="d-flex flex-column h-100">
                        <div class="flex-grow-1 overflow-auto p-10 d-flex flex-center">
                            <div class="manual-container position-relative" style="max-width: 1100px; width: 100%; min-height: 400px;">
                                <!-- Loader -->
                                <div class="pdf-loader spinner-border text-primary" role="status">
                                    <span class="visually-hidden">Loading...</span>
                                </div>

                                <!-- Cover Page -->
                                <img src="{{ asset('assets/media/manuals/manual_cover.png') }}" class="manual-page-img active" id="page_1">
                                
                                <!-- PDF Canvas Container -->
                                <div class="manual-pdf-container" id="pdf-container">
                                    <canvas id="pdf-render"></canvas>
                                </div>
                            </div>
                        </div>
                        <div class="bg-white py-4 px-10 d-flex justify-content-between align-items-center shadow-sm">
                            <button class="btn btn-secondary btn-sm" id="prevPage" disabled>
                                <i class="ki-outline ki-left fs-2"></i> Sebelumnya
                            </button>
                            <span class="fw-bold fs-5 text-gray-700" id="pageNum">Loading Panduan...</span>
                            <button class="btn btn-primary btn-sm" id="nextPage" disabled>
                                Selanjutnya <i class="ki-outline ki-right fs-2"></i>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    @push('scripts')
        <script>
            const togglePassword = document.querySelector('#togglePassword');
            const passwordInput = document.querySelector('#passwordInput');
            const toggleIcon = document.querySelector('#toggleIcon');

            if (togglePassword && passwordInput) {
                togglePassword.addEventListener('click', function(e) {
                    // Toggle type attribute
                    const type = passwordInput.getAttribute('type') === 'password' ? 'text' : 'password';
                    passwordInput.setAttribute('type', type);

                    // Toggle icon class (Metronic Icons)
                    if (type === 'text') {
                        toggleIcon.classList.remove('ki-eye-slash');
                        toggleIcon.classList.add('ki-eye');
                    } else {
                        toggleIcon.classList.remove('ki-eye');
                        toggleIcon.classList.add('ki-eye-slash');
                    }
                });
            }
        </script>

        <script>
            document.getElementById('kt_sign_in_form').addEventListener('submit', function(e) {
                e.preventDefault();

                // 1. Ambil Elemen Tombol
                const submitButton = document.getElementById('kt_sign_in_submit');

                // 2. Aktifkan Loading State
                submitButton.setAttribute("data-kt-indicator", "on");
                submitButton.classList.add("disabled");
                submitButton.disabled = true;

                const label = submitButton.querySelector('.indicator-label');
                const progress = submitButton.querySelector('.indicator-progress');
                if (label) label.style.display = 'none';
                if (progress) progress.style.display = 'inline-block';

                let formData = new FormData(this);

                fetch("{{ route('login') }}", {
                        method: "POST",
                        headers: {
                            "X-CSRF-TOKEN": "{{ csrf_token() }}",
                            "Accept": "application/json"
                        },
                        body: formData
                    })
                    .then(async response => {
                        let result = await response.json();

                        if (!response.ok) {
                            // Reset Tombol
                            submitButton.removeAttribute("data-kt-indicator");
                            submitButton.classList.remove("disabled");
                            submitButton.disabled = false;
                            if (label) label.style.display = 'block';
                            if (progress) progress.style.display = 'none';

                            // === KASUS 1: LOCKOUT (Status 429) ===
                            if (response.status === 429) {
                                let seconds = 60;
                                if (result.errors && result.errors.seconds) {
                                    seconds = result.errors.seconds[0];
                                } else if (result.errors && result.errors.email) {
                                    let match = result.errors.email[0].match(/(\d+)/);
                                    if (match) seconds = match[0];
                                }
                                showLockoutCountdown(seconds);
                                return;
                            }

                            // === KASUS 2: ERROR BIASA ===
                            let errorMessage = "Terjadi kesalahan sistem.";
                            if (result.errors && result.errors.email) {
                                errorMessage = result.errors.email[0];
                            } else if (result.message) {
                                errorMessage = result.message;
                            }

                            Swal.fire({
                                icon: "error",
                                title: "Login Gagal!",
                                text: errorMessage,
                                confirmButtonColor: "#d33",
                                buttonsStyling: false,
                                customClass: {
                                    confirmButton: "btn btn-danger"
                                }
                            });

                            return;
                        }

                        // 4. JIKA SUKSES
                        let redirectUrl = result.redirect || "{{ route('dashboard') }}";
                        superPremiumThreeDotLoader(redirectUrl);
                    })
                    .catch(err => {
                        console.error("Fetch Error:", err);
                        submitButton.removeAttribute("data-kt-indicator");
                        submitButton.disabled = false;
                        if (label) label.style.display = 'block';
                        if (progress) progress.style.display = 'none';

                        Swal.fire({
                            icon: "error",
                            title: "Error Jaringan",
                            text: "Tidak dapat terhubung ke server.",
                            confirmButtonColor: "#d33"
                        });
                    });
            });

            // ==========================================
            // FUNGSI 1: COUNTDOWN LOCKOUT
            // ==========================================
            function showLockoutCountdown(seconds) {
                let originalSeconds = seconds;
                const submitButton = document.getElementById('kt_sign_in_submit');
                submitButton.disabled = true;

                Swal.fire({
                    icon: "warning",
                    title: "Terlalu Banyak Percobaan!",
                    html: `
                    Anda telah gagal login berulang kali.<br>
                    Coba lagi dalam <b id="countdown" class="text-danger fs-1">${seconds}</b> detik.
                    <br><br>
                    <div class="progress bg-secondary" style="height: 10px; border-radius: 20px;">
                        <div id="lock-progress" class="progress-bar bg-danger" style="width: 100%; transition: width 1s linear;"></div>
                    </div>
                `,
                    allowOutsideClick: false,
                    allowEscapeKey: false,
                    showConfirmButton: false,
                    timer: seconds * 1000,
                    didOpen: () => {
                        let countdownEl = document.getElementById("countdown");
                        let bar = document.getElementById("lock-progress");

                        let interval = setInterval(() => {
                            seconds--;
                            if (countdownEl) countdownEl.textContent = seconds;
                            if (bar) {
                                let percent = Math.floor((seconds / originalSeconds) * 100);
                                bar.style.width = percent + "%";
                            }
                            if (seconds <= 0) {
                                clearInterval(interval);
                                submitButton.disabled = false;
                                submitButton.classList.remove("disabled");
                            }
                        }, 1000);
                    }
                });
            }

            // ==========================================
            // FUNGSI 2: SUPER PREMIUM LOADER
            // ==========================================
            function superPremiumThreeDotLoader(targetUrl) {
                let timerInterval;
                if (!document.getElementById('dot-loader-style')) {
                    const styleDots = document.createElement('style');
                    styleDots.id = 'dot-loader-style';
                    styleDots.textContent = `
                    .dot-loader { width: 12px; height: 12px; background-color: #22c55e; border-radius: 50%; animation: bounceDot 0.6s infinite alternate; }
                    .dot-loader--2 { animation-delay: 0.15s; }
                    .dot-loader--3 { animation-delay: 0.3s; }
                    @keyframes bounceDot { 0% { transform: translateY(0); opacity: 1; } 100% { transform: translateY(-10px); opacity: 0.4; } }
                `;
                    document.head.appendChild(styleDots);
                }

                Swal.fire({
                    icon: "success",
                    title: `<span class="fw-bold">Login Berhasil</span>`,
                    html: `
                    <div class="text-muted mb-3">Menyiapkan aplikasi untuk Anda...</div>
                    <div class="my-12" style="display:flex; justify-content:center; align-items:center; gap:10px; margin-bottom:22px;">
                        <div class="dot-loader"></div>
                        <div class="dot-loader dot-loader--2"></div>
                        <div class="dot-loader dot-loader--3"></div>
                    </div>
                    <div class='progress bg-secondary mt-3' style='height: 12px; border-radius: 20px; width: 100%; overflow: hidden;'>
                        <div id="sa-progress-premium" class='progress-bar bg-success' style='width: 0%; border-radius: 20px'></div>
                    </div>
                    <div id="sa-percent" class="mt-2 fw-bold text-gray-700">0%</div>
                `,
                    width: 400,
                    padding: "2em",
                    timer: 2000,
                    showConfirmButton: false,
                    allowOutsideClick: false,
                    didOpen: () => {
                        let bar = document.getElementById("sa-progress-premium");
                        let percentText = document.getElementById("sa-percent");
                        let width = 0;
                        timerInterval = setInterval(() => {
                            width += Math.floor(Math.random() * 5) + 1;
                            if (width > 100) width = 100;
                            bar.style.width = width + "%";
                            percentText.innerHTML = width + "%";
                            if (width >= 100) clearInterval(timerInterval);
                        }, 50);
                    },
                    willClose: () => {
                        clearInterval(timerInterval);
                    }
                }).then(() => {
                    const container = document.querySelector(".d-flex.flex-column-fluid");
                    if (container) {
                        container.style.opacity = 0;
                        container.style.transition = "opacity .5s ease-in-out";
                    }
                    setTimeout(() => {
                        window.location.href = targetUrl;
                    }, 300);
                });
            }

            // --- MANUAL BOOK VIEWER LOGIC (PDF.JS INTEGRATION) ---
            const pdfUrl = "{{ asset('assets/media/manuals/Panduan_DineSyncPOS.pdf') }}";
            let pdfDoc = null;
            let currentPage = 1;
            let totalPages = 1;
            let pageRendering = false;
            let pageNumPending = null;

            // Load PDF library
            const scriptPdf = document.createElement('script');
            scriptPdf.src = "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js";
            document.head.appendChild(scriptPdf);

            scriptPdf.onload = () => {
                pdfjsLib.GlobalWorkerOptions.workerSrc = "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js";
                loadPdf();
            };

            function loadPdf() {
                pdfjsLib.getDocument(pdfUrl).promise.then(pdf => {
                    pdfDoc = pdf;
                    totalPages = pdf.numPages + 1; // +1 karena halaman 1 adalah Cover Image
                    document.getElementById('nextPage').disabled = false;
                    updatePage();
                }).catch(err => {
                    console.error("PDF Load Error:", err);
                    document.getElementById('pageNum').textContent = "Gagal memuat PDF.";
                });
            }

            function renderPdfPage(num) {
                pageRendering = true;
                const loader = document.querySelector('.pdf-loader');
                loader.style.display = 'block';

                pdfDoc.getPage(num).then(page => {
                    const viewport = page.getViewport({ scale: 3 }); // Scale 3 untuk kualitas HD ultra-tajam
                    const canvas = document.getElementById('pdf-render');
                    const ctx = canvas.getContext('2d');
                    
                    canvas.height = viewport.height;
                    canvas.width = viewport.width;

                    const renderContext = {
                        canvasContext: ctx,
                        viewport: viewport
                    };

                    const renderTask = page.render(renderContext);
                    renderTask.promise.then(() => {
                        pageRendering = false;
                        loader.style.display = 'none';
                        if (pageNumPending !== null) {
                            renderPdfPage(pageNumPending);
                            pageNumPending = null;
                        }
                    });
                });
            }

            function openViewer() {
                currentPage = 1;
                updatePage();
                $("#modal_manual_book").modal('hide');
                setTimeout(() => {
                    $("#modal_manual_viewer").modal('show');
                }, 400);
            }

            function updatePage() {
                // Sembunyikan semua konten
                document.getElementById('page_1').classList.remove('active');
                document.getElementById('pdf-container').classList.remove('active');

                if (currentPage === 1) {
                    document.getElementById('page_1').classList.add('active');
                    document.getElementById('pageNum').textContent = `Sampul Panduan (Halaman 1 / ${totalPages})`;
                } else {
                    document.getElementById('pdf-container').classList.add('active');
                    document.getElementById('pageNum').textContent = `Panduan Digital (Halaman ${currentPage} / ${totalPages})`;
                    
                    // Render halaman PDF (pdfPage = currentPage - 1)
                    const pdfPageNum = currentPage - 1;
                    if (pageRendering) {
                        pageNumPending = pdfPageNum;
                    } else {
                        renderPdfPage(pdfPageNum);
                    }
                }
                
                // Update button state
                document.getElementById('prevPage').disabled = (currentPage === 1);
                document.getElementById('nextPage').disabled = (currentPage === totalPages);
            }

            document.getElementById('nextPage').addEventListener('click', () => {
                if (currentPage < totalPages) {
                    currentPage++;
                    updatePage();
                }
            });

            document.getElementById('prevPage').addEventListener('click', () => {
                if (currentPage > 1) {
                    currentPage--;
                    updatePage();
                }
            });
        </script>
    @endpush
@endsection
