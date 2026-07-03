@extends('backend.layout.app')
@section('title', 'Dashboard')
@section('content')

    <div class="mt-5 mb-10">

         <!--begin::Row-->
							<div class="row gy-0 gx-10">
								<!--begin::Col-->
								<div class="col-xl-8">
									<!--begin::General Widget 1-->
									<div class="mb-10">
										<!--begin::Tabs-->
										<ul class="nav row mb-10">
											<li class="nav-item col-12 col-lg mb-5 mb-lg-0">
												<a class="nav-link btn btn-flex btn-color-gray-500 btn-outline btn-active-primary d-flex flex-grow-1 flex-column flex-center py-5 h-1250px h-lg-175px" data-bs-toggle="tab" href="#kt_general_widget_1_1">
													<i class="ki-duotone ki-abstract-26 fs-2x mb-5 mx-0">
														<span class="path1"></span>
														<span class="path2"></span>
													</i>
													<span class="fs-6 fw-bold">SaaS 
													<br />Application</span>
												</a>
											</li>
											<li class="nav-item col-12 col-lg mb-5 mb-lg-0">
												<a class="nav-link btn btn-flex btn-color-gray-500 btn-outline btn-active-primary d-flex flex-grow-1 flex-column flex-center py-5 h-1250px h-lg-175px" data-bs-toggle="tab" href="#kt_general_widget_1_2">
													<i class="ki-duotone ki-element-11 fs-2x mb-5 mx-0">
														<span class="path1"></span>
														<span class="path2"></span>
														<span class="path3"></span>
														<span class="path4"></span>
													</i>
													<span class="fs-6 fw-bold">Main 
													<br />Categories</span>
												</a>
											</li>
											<li class="nav-item col-12 col-lg mb-5 mb-lg-0">
												<a class="nav-link btn btn-flex btn-color-gray-500 btn-outline btn-active-primary d-flex flex-grow-1 flex-column flex-center py-5 h-1250px h-lg-175px active" data-bs-toggle="tab" href="#kt_general_widget_1_3">
													<i class="ki-duotone ki-briefcase fs-2x mb-5 mx-0">
														<span class="path1"></span>
														<span class="path2"></span>
													</i>
													<span class="fs-6 fw-bold">Order 
													<br />Management</span>
												</a>
											</li>
											<li class="nav-item col-12 col-lg mb-5 mb-lg-0">
												<a class="nav-link btn btn-flex btn-color-gray-500 btn-outline btn-active-primary d-flex flex-grow-1 flex-column flex-center py-5 h-1250px h-lg-175px" data-bs-toggle="tab" href="#kt_general_widget_1_4">
													<i class="ki-duotone ki-chart-simple fs-2x mb-5 mx-0">
														<span class="path1"></span>
														<span class="path2"></span>
														<span class="path3"></span>
														<span class="path4"></span>
													</i>
													<span class="fs-6 fw-bold">Sales 
													<br />Statistics</span>
												</a>
											</li>
											<li class="nav-item col-12 col-lg mb-5 mb-lg-0">
												<a class="nav-link btn btn-flex btn-color-gray-500 btn-outline btn-active-primary d-flex flex-grow-1 flex-column flex-center py-5 h-1250px h-lg-175px" data-bs-toggle="tab" href="#kt_general_widget_1_5">
													<i class="ki-duotone ki-shield-tick fs-2x mb-5 mx-0">
														<span class="path1"></span>
														<span class="path2"></span>
													</i>
													<span class="fs-6 fw-bold">Access 
													<br />Control</span>
												</a>
											</li>
										</ul>
										<!--begin::Tab content-->
										<div class="tab-content">
											<div class="tab-pane fade" id="kt_general_widget_1_1">
												<!--begin::Tables Widget 2-->
												<div class="card">
													<!--begin::Header-->
													<div class="card-header border-0 pt-5">
														<h3 class="card-title align-items-start flex-column">
															<span class="card-label fw-bold fs-3 mb-1">Latest Arrivals</span>
															<span class="text-muted mt-1 fw-semibold fs-7">More than 100 new products</span>
														</h3>
														<div class="card-toolbar">
															<!--begin::Menu-->
															<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
																<i class="ki-duotone ki-category fs-6">
																	<span class="path1"></span>
																	<span class="path2"></span>
																	<span class="path3"></span>
																	<span class="path4"></span>
																</i>
															</button>
															<!--begin::Menu 1-->
															<div class="menu menu-sub menu-sub-dropdown w-250px w-md-300px" data-kt-menu="true" id="kt_menu_66b9a49b3f381">
																<!--begin::Header-->
																<div class="px-7 py-5">
																	<div class="fs-5 text-gray-900 fw-bold">Filter Options</div>
																</div>
																<!--end::Header-->
																<!--begin::Menu separator-->
																<div class="separator border-gray-200"></div>
																<!--end::Menu separator-->
																<!--begin::Form-->
																<div class="px-7 py-5">
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Status:</label>
																		<!--end::Label-->
																		<!--begin::Input-->
																		<div>
																			<select class="form-select form-select-solid" multiple="multiple" data-kt-select2="true" data-close-on-select="false" data-placeholder="Select option" data-dropdown-parent="#kt_menu_66b9a49b3f381" data-allow-clear="true">
																				<option></option>
																				<option value="1">Approved</option>
																				<option value="2">Pending</option>
																				<option value="2">In Process</option>
																				<option value="2">Rejected</option>
																			</select>
																		</div>
																		<!--end::Input-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Member Type:</label>
																		<!--end::Label-->
																		<!--begin::Options-->
																		<div class="d-flex">
																			<!--begin::Options-->
																			<label class="form-check form-check-sm form-check-custom form-check-solid me-5">
																				<input class="form-check-input" type="checkbox" value="1" />
																				<span class="form-check-label">Author</span>
																			</label>
																			<!--end::Options-->
																			<!--begin::Options-->
																			<label class="form-check form-check-sm form-check-custom form-check-solid">
																				<input class="form-check-input" type="checkbox" value="2" checked="checked" />
																				<span class="form-check-label">Customer</span>
																			</label>
																			<!--end::Options-->
																		</div>
																		<!--end::Options-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Notifications:</label>
																		<!--end::Label-->
																		<!--begin::Switch-->
																		<div class="form-check form-switch form-switch-sm form-check-custom form-check-solid">
																			<input class="form-check-input" type="checkbox" value="" name="notifications" checked="checked" />
																			<label class="form-check-label">Enabled</label>
																		</div>
																		<!--end::Switch-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Actions-->
																	<div class="d-flex justify-content-end">
																		<button type="reset" class="btn btn-sm btn-light btn-active-light-primary me-2" data-kt-menu-dismiss="true">Reset</button>
																		<button type="submit" class="btn btn-sm btn-primary" data-kt-menu-dismiss="true">Apply</button>
																	</div>
																	<!--end::Actions-->
																</div>
																<!--end::Form-->
															</div>
															<!--end::Menu 1-->
															<!--end::Menu-->
														</div>
													</div>
													<!--end::Header-->
													<!--begin::Body-->
													<div class="card-body py-3">
														<!--begin::Table container-->
														<div class="table-responsive">
															<!--begin::Table-->
															<table class="table align-middle gs-0 gy-5">
																<!--begin::Table head-->
																<thead>
																	<tr>
																		<th class="p-0 w-50px"></th>
																		<th class="p-0 min-w-150px"></th>
																		<th class="p-0 min-w-150px"></th>
																		<th class="p-0 min-w-125px"></th>
																		<th class="p-0 min-w-40px"></th>
																	</tr>
																</thead>
																<!--end::Table head-->
																<!--begin::Table body-->
																<tbody>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/plurk.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Top Authors</a>
																			<span class="text-muted fw-semibold d-block fs-7">Successful Fellas</span>
																		</td>
																		<td class="text-end">
																			<span class="badge badge-light-danger fw-semibold me-1">Angular</span>
																			<span class="badge badge-light-info fw-semibold me-1">PHP</span>
																		</td>
																		<td class="text-end">
																			<span class="text-muted fw-bold">4600 Users</span>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																			<span class="text-muted fw-semibold d-block fs-7">Most Successful</span>
																		</td>
																		<td class="text-end">
																			<span class="badge badge-light-danger fw-semibold me-1">HTML</span>
																			<span class="badge badge-light-info fw-semibold me-1">CSS</span>
																		</td>
																		<td class="text-end">
																			<span class="text-muted fw-bold">7200 Users</span>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/vimeo.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">New Users</a>
																			<span class="text-muted fw-semibold d-block fs-7">Awesome Users</span>
																		</td>
																		<td class="text-end">
																			<span class="badge badge-light-danger fw-semibold me-1">React</span>
																			<span class="badge badge-light-info fw-semibold me-1">SASS</span>
																		</td>
																		<td class="text-end">
																			<span class="text-muted fw-bold">890 Users</span>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																			<span class="text-muted fw-semibold d-block fs-7">Best Customers</span>
																		</td>
																		<td class="text-end">
																			<span class="badge badge-light-danger fw-semibold me-1">Java</span>
																			<span class="badge badge-light-info fw-semibold me-1">PHP</span>
																		</td>
																		<td class="text-end">
																			<span class="text-muted fw-bold">6370 Users</span>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/kickstarter.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Bestseller Theme</a>
																			<span class="text-muted fw-semibold d-block fs-7">Amazing Templates</span>
																		</td>
																		<td class="text-end">
																			<span class="badge badge-light-danger fw-semibold me-1">Python</span>
																			<span class="badge badge-light-info fw-semibold me-1">MySQL</span>
																		</td>
																		<td class="text-end">
																			<span class="text-muted fw-bold">354 Users</span>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																</tbody>
																<!--end::Table body-->
															</table>
															<!--end::Table-->
														</div>
														<!--end::Table container-->
													</div>
													<!--end::Body-->
												</div>
												<!--end::Tables Widget 2-->
											</div>
											<div class="tab-pane fade" id="kt_general_widget_1_2">
												<!--begin::Tables Widget 3-->
												<div class="card">
													<!--begin::Header-->
													<div class="card-header border-0 pt-5">
														<h3 class="card-title align-items-start flex-column">
															<span class="card-label fw-bold fs-3 mb-1">Files</span>
															<span class="text-muted mt-1 fw-semibold fs-7">Over 100 pending files</span>
														</h3>
														<div class="card-toolbar">
															<!--begin::Menu-->
															<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
																<i class="ki-duotone ki-category fs-6">
																	<span class="path1"></span>
																	<span class="path2"></span>
																	<span class="path3"></span>
																	<span class="path4"></span>
																</i>
															</button>
															<!--begin::Menu 3-->
															<div class="menu menu-sub menu-sub-dropdown menu-column menu-rounded menu-gray-800 menu-state-bg-light-primary fw-semibold w-200px py-3" data-kt-menu="true">
																<!--begin::Heading-->
																<div class="menu-item px-3">
																	<div class="menu-content text-muted pb-2 px-3 fs-7 text-uppercase">Payments</div>
																</div>
																<!--end::Heading-->
																<!--begin::Menu item-->
																<div class="menu-item px-3">
																	<a href="#" class="menu-link px-3">Create Invoice</a>
																</div>
																<!--end::Menu item-->
																<!--begin::Menu item-->
																<div class="menu-item px-3">
																	<a href="#" class="menu-link flex-stack px-3">Create Payment 
																	<span class="ms-2" data-bs-toggle="tooltip" title="Specify a target name for future usage and reference">
																		<i class="ki-duotone ki-information fs-6">
																			<span class="path1"></span>
																			<span class="path2"></span>
																			<span class="path3"></span>
																		</i>
																	</span></a>
																</div>
																<!--end::Menu item-->
																<!--begin::Menu item-->
																<div class="menu-item px-3">
																	<a href="#" class="menu-link px-3">Generate Bill</a>
																</div>
																<!--end::Menu item-->
																<!--begin::Menu item-->
																<div class="menu-item px-3" data-kt-menu-trigger="hover" data-kt-menu-placement="right-end">
																	<a href="#" class="menu-link px-3">
																		<span class="menu-title">Subscription</span>
																		<span class="menu-arrow"></span>
																	</a>
																	<!--begin::Menu sub-->
																	<div class="menu-sub menu-sub-dropdown w-175px py-4">
																		<!--begin::Menu item-->
																		<div class="menu-item px-3">
																			<a href="#" class="menu-link px-3">Plans</a>
																		</div>
																		<!--end::Menu item-->
																		<!--begin::Menu item-->
																		<div class="menu-item px-3">
																			<a href="#" class="menu-link px-3">Billing</a>
																		</div>
																		<!--end::Menu item-->
																		<!--begin::Menu item-->
																		<div class="menu-item px-3">
																			<a href="#" class="menu-link px-3">Statements</a>
																		</div>
																		<!--end::Menu item-->
																		<!--begin::Menu separator-->
																		<div class="separator my-2"></div>
																		<!--end::Menu separator-->
																		<!--begin::Menu item-->
																		<div class="menu-item px-3">
																			<div class="menu-content px-3">
																				<!--begin::Switch-->
																				<label class="form-check form-switch form-check-custom form-check-solid">
																					<!--begin::Input-->
																					<input class="form-check-input w-30px h-20px" type="checkbox" value="1" checked="checked" name="notifications" />
																					<!--end::Input-->
																					<!--end::Label-->
																					<span class="form-check-label text-muted fs-6">Recuring</span>
																					<!--end::Label-->
																				</label>
																				<!--end::Switch-->
																			</div>
																		</div>
																		<!--end::Menu item-->
																	</div>
																	<!--end::Menu sub-->
																</div>
																<!--end::Menu item-->
																<!--begin::Menu item-->
																<div class="menu-item px-3 my-1">
																	<a href="#" class="menu-link px-3">Settings</a>
																</div>
																<!--end::Menu item-->
															</div>
															<!--end::Menu 3-->
															<!--end::Menu-->
														</div>
													</div>
													<!--end::Header-->
													<!--begin::Body-->
													<div class="card-body py-3">
														<!--begin::Table container-->
														<div class="table-responsive">
															<!--begin::Table-->
															<table class="table align-middle gs-0 gy-3">
																<!--begin::Table head-->
																<thead>
																	<tr>
																		<th class="p-0 w-50px"></th>
																		<th class="p-0 min-w-150px"></th>
																		<th class="p-0 min-w-140px"></th>
																		<th class="p-0 min-w-120px"></th>
																		<th class="p-0 min-w-40px"></th>
																	</tr>
																</thead>
																<!--end::Table head-->
																<!--begin::Table body-->
																<tbody>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label bg-light-success">
																					<i class="ki-duotone ki-basket fs-2x text-success">
																						<span class="path1"></span>
																						<span class="path2"></span>
																						<span class="path3"></span>
																						<span class="path4"></span>
																					</i>
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Top Authors</a>
																		</td>
																		<td class="text-end text-muted fw-bold">ReactJs, HTML</td>
																		<td class="text-end text-muted fw-bold">4600 Users</td>
																		<td class="text-end text-gray-900 fw-bold fs-6 pe-0">5.4MB</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label bg-light-danger">
																					<i class="ki-duotone ki-element-11 fs-2x text-danger">
																						<span class="path1"></span>
																						<span class="path2"></span>
																						<span class="path3"></span>
																						<span class="path4"></span>
																					</i>
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																		</td>
																		<td class="text-end text-muted fw-bold">Python, MySQL</td>
																		<td class="text-end text-muted fw-bold">7200 Users</td>
																		<td class="text-end text-gray-900 fw-bold fs-6 pe-0">2.8MB</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label bg-light-info">
																					<i class="ki-duotone ki-briefcase fs-2x text-info">
																						<span class="path1"></span>
																						<span class="path2"></span>
																					</i>
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">New Users</a>
																		</td>
																		<td class="text-end text-muted fw-bold">Laravel, Metronic</td>
																		<td class="text-end text-muted fw-bold">890 Users</td>
																		<td class="text-end text-gray-900 fw-bold fs-6 pe-0">1.5MB</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label bg-light-warning">
																					<i class="ki-duotone ki-abstract-26 fs-2x text-warning">
																						<span class="path1"></span>
																						<span class="path2"></span>
																					</i>
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																		</td>
																		<td class="text-end text-muted fw-bold">AngularJS, C#</td>
																		<td class="text-end text-muted fw-bold">4600 Users</td>
																		<td class="text-end text-gray-900 fw-bold fs-6 pe-0">5.4MB</td>
																	</tr>
																	<tr>
																		<td>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label bg-light-primary">
																					<i class="ki-duotone ki-abstract-41 fs-2x text-primary">
																						<span class="path1"></span>
																						<span class="path2"></span>
																					</i>
																				</span>
																			</div>
																		</td>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																		</td>
																		<td class="text-end text-muted fw-bold">ReactJS, Ruby</td>
																		<td class="text-end text-muted fw-bold">354 Users</td>
																		<td class="text-end text-gray-900 fw-bold fs-6 pe-0">500KB</td>
																	</tr>
																</tbody>
																<!--end::Table body-->
															</table>
															<!--end::Table-->
														</div>
														<!--end::Table container-->
													</div>
													<!--begin::Body-->
												</div>
												<!--end::Tables Widget 3-->
											</div>
											<div class="tab-pane fade show active" id="kt_general_widget_1_3">
												<!--begin::Tables Widget 5-->
												<div class="card">
													<!--begin::Header-->
													<div class="card-header border-0 pt-5">
														<h3 class="card-title align-items-start flex-column">
															<span class="card-label fw-bold fs-3 mb-1">Latest Products</span>
															<span class="text-muted mt-1 fw-semibold fs-7">More than 400 new products</span>
														</h3>
														<div class="card-toolbar">
															<ul class="nav">
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-secondary fw-bold px-4 me-1 active" data-bs-toggle="tab" href="#kt_table_widget_5_tab_1">Month</a>
																</li>
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-secondary fw-bold px-4 me-1" data-bs-toggle="tab" href="#kt_table_widget_5_tab_2">Week</a>
																</li>
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-secondary fw-bold px-4" data-bs-toggle="tab" href="#kt_table_widget_5_tab_3">Day</a>
																</li>
															</ul>
														</div>
													</div>
													<!--end::Header-->
													<!--begin::Body-->
													<div class="card-body py-3">
														<div class="tab-content">
															<!--begin::Tap pane-->
															<div class="tab-pane fade show active" id="kt_table_widget_5_tab_1">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table table-row-dashed table-row-gray-200 align-middle gs-0 gy-4">
																		<!--begin::Table head-->
																		<thead>
																			<tr class="border-0">
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-110px"></th>
																				<th class="p-0 min-w-50px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/plurk.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Brad Simmons</a>
																					<span class="text-muted fw-semibold d-block">Movie Creator</span>
																				</td>
																				<td class="text-end text-muted fw-bold">React, HTML</td>
																				<td class="text-end">
																					<span class="badge badge-light-success">Approved</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																					<span class="text-muted fw-semibold d-block">Most Successful</span>
																				</td>
																				<td class="text-end text-muted fw-bold">Python, MySQL</td>
																				<td class="text-end">
																					<span class="badge badge-light-warning">In Progress</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/vimeo.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">New Users</a>
																					<span class="text-muted fw-semibold d-block">Awesome Users</span>
																				</td>
																				<td class="text-end text-muted fw-bold">Laravel,Metronic</td>
																				<td class="text-end">
																					<span class="badge badge-light-primary">Success</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																					<span class="text-muted fw-semibold d-block">Movie Creator</span>
																				</td>
																				<td class="text-end text-muted fw-bold">AngularJS, C#</td>
																				<td class="text-end">
																					<span class="badge badge-light-danger">Rejected</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/kickstarter.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Bestseller Theme</a>
																					<span class="text-muted fw-semibold d-block">Best Customers</span>
																				</td>
																				<td class="text-end text-muted fw-bold">ReactJS, Ruby</td>
																				<td class="text-end">
																					<span class="badge badge-light-warning">In Progress</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
															<!--begin::Tap pane-->
															<div class="tab-pane fade" id="kt_table_widget_5_tab_2">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table table-row-dashed table-row-gray-200 align-middle gs-0 gy-4">
																		<!--begin::Table head-->
																		<thead>
																			<tr class="border-0">
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-110px"></th>
																				<th class="p-0 min-w-50px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/plurk.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Brad Simmons</a>
																					<span class="text-muted fw-semibold d-block">Movie Creator</span>
																				</td>
																				<td class="text-end text-muted fw-bold">React, HTML</td>
																				<td class="text-end">
																					<span class="badge badge-light-success">Approved</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																					<span class="text-muted fw-semibold d-block">Most Successful</span>
																				</td>
																				<td class="text-end text-muted fw-bold">Python, MySQL</td>
																				<td class="text-end">
																					<span class="badge badge-light-warning">In Progress</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																					<span class="text-muted fw-semibold d-block">Movie Creator</span>
																				</td>
																				<td class="text-end text-muted fw-bold">AngularJS, C#</td>
																				<td class="text-end">
																					<span class="badge badge-light-danger">Rejected</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
															<!--begin::Tap pane-->
															<div class="tab-pane fade" id="kt_table_widget_5_tab_3">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table table-row-dashed table-row-gray-200 align-middle gs-0 gy-4">
																		<!--begin::Table head-->
																		<thead>
																			<tr class="border-0">
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-110px"></th>
																				<th class="p-0 min-w-50px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/kickstarter.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Bestseller Theme</a>
																					<span class="text-muted fw-semibold d-block">Best Customers</span>
																				</td>
																				<td class="text-end text-muted fw-bold">ReactJS, Ruby</td>
																				<td class="text-end">
																					<span class="badge badge-light-warning">In Progress</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																					<span class="text-muted fw-semibold d-block">Movie Creator</span>
																				</td>
																				<td class="text-end text-muted fw-bold">AngularJS, C#</td>
																				<td class="text-end">
																					<span class="badge badge-light-danger">Rejected</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/vimeo.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">New Users</a>
																					<span class="text-muted fw-semibold d-block">Awesome Users</span>
																				</td>
																				<td class="text-end text-muted fw-bold">Laravel,Metronic</td>
																				<td class="text-end">
																					<span class="badge badge-light-primary">Success</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-45px me-2">
																						<span class="symbol-label">
																							<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
																						</span>
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																					<span class="text-muted fw-semibold d-block">Most Successful</span>
																				</td>
																				<td class="text-end text-muted fw-bold">Python, MySQL</td>
																				<td class="text-end">
																					<span class="badge badge-light-warning">In Progress</span>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																						<i class="ki-duotone ki-arrow-right fs-2">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
														</div>
													</div>
													<!--end::Body-->
												</div>
												<!--end::Tables Widget 5-->
											</div>
											<div class="tab-pane fade" id="kt_general_widget_1_4">
												<!--begin::Tables Widget 4-->
												<div class="card">
													<!--begin::Header-->
													<div class="card-header border-0 pt-5">
														<h3 class="card-title align-items-start flex-column">
															<span class="card-label fw-bold fs-3 mb-1">New Members</span>
															<span class="text-muted mt-1 fw-semibold fs-7">More than 400 new members</span>
														</h3>
														<div class="card-toolbar">
															<ul class="nav">
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-light-primary active fw-bold px-4 me-1" data-bs-toggle="tab" href="#kt_table_widget_4_tab_1">Month</a>
																</li>
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-light-primary fw-bold px-4 me-1" data-bs-toggle="tab" href="#kt_table_widget_4_tab_2">Week</a>
																</li>
																<li class="nav-item">
																	<a class="nav-link btn btn-sm btn-color-muted btn-active btn-active-light-primary fw-bold px-4" data-bs-toggle="tab" href="#kt_table_widget_4_tab_3">Day</a>
																</li>
															</ul>
														</div>
													</div>
													<!--end::Header-->
													<!--begin::Body-->
													<div class="card-body py-3">
														<div class="tab-content">
															<!--begin::Tap pane-->
															<div class="tab-pane fade show active" id="kt_table_widget_4_tab_1">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table align-middle gs-0 gy-3">
																		<!--begin::Table head-->
																		<thead>
																			<tr>
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-120px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/avatars/300-14.jpg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Brad Simmons</a>
																					<span class="text-muted fw-semibold d-block fs-7">Movie Creator</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/avatars/300-5.jpg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Jessie Clarcson</a>
																					<span class="text-muted fw-semibold d-block fs-7">HTML, CSS Coding</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/avatars/300-20.jpg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Lebron Wayde</a>
																					<span class="text-muted fw-semibold d-block fs-7">ReactJS Developer</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/avatars/300-23.jpg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Natali Trump</a>
																					<span class="text-muted fw-semibold d-block fs-7">UI/UX Designer</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/avatars/300-10.jpg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Kevin Leonard</a>
																					<span class="text-muted fw-semibold d-block fs-7">Art Director</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
															<!--begin::Tap pane-->
															<div class="tab-pane fade" id="kt_table_widget_4_tab_2">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table align-middle gs-0 gy-3">
																		<!--begin::Table head-->
																		<thead>
																			<tr>
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-120px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/043-boy-18.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Kevin Leonard</a>
																					<span class="text-muted fw-semibold d-block fs-7">Art Director</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/014-girl-7.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Natali Trump</a>
																					<span class="text-muted fw-semibold d-block fs-7">UI/UX Designer</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/018-girl-9.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Jessie Clarcson</a>
																					<span class="text-muted fw-semibold d-block fs-7">HTML, CSS Coding</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/001-boy.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Brad Simmons</a>
																					<span class="text-muted fw-semibold d-block fs-7">Movie Creator</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
															<!--begin::Tap pane-->
															<div class="tab-pane fade" id="kt_table_widget_4_tab_3">
																<!--begin::Table container-->
																<div class="table-responsive">
																	<!--begin::Table-->
																	<table class="table align-middle gs-0 gy-3">
																		<!--begin::Table head-->
																		<thead>
																			<tr>
																				<th class="p-0 w-50px"></th>
																				<th class="p-0 min-w-150px"></th>
																				<th class="p-0 min-w-140px"></th>
																				<th class="p-0 min-w-120px"></th>
																			</tr>
																		</thead>
																		<!--end::Table head-->
																		<!--begin::Table body-->
																		<tbody>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/018-girl-9.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Jessie Clarcson</a>
																					<span class="text-muted fw-semibold d-block fs-7">HTML, CSS Coding</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/047-girl-25.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Lebron Wayde</a>
																					<span class="text-muted fw-semibold d-block fs-7">ReactJS Developer</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																			<tr>
																				<td>
																					<div class="symbol symbol-50px">
																						<img src="assets/media/svg/avatars/014-girl-7.svg" alt="" />
																					</div>
																				</td>
																				<td>
																					<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Natali Trump</a>
																					<span class="text-muted fw-semibold d-block fs-7">UI/UX Designer</span>
																				</td>
																				<td>
																					<span class="text-muted fw-semibold d-block fs-7">Rating</span>
																					<div class="rating">
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																						<div class="rating-label checked">
																							<i class="ki-duotone ki-star fs-6"></i>
																						</div>
																					</div>
																				</td>
																				<td class="text-end">
																					<a href="#" class="btn btn-icon btn-light-twitter btn-sm me-3">
																						<i class="ki-duotone ki-twitter fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																					<a href="#" class="btn btn-icon btn-light-facebook btn-sm">
																						<i class="ki-duotone ki-facebook fs-4">
																							<span class="path1"></span>
																							<span class="path2"></span>
																						</i>
																					</a>
																				</td>
																			</tr>
																		</tbody>
																		<!--end::Table body-->
																	</table>
																</div>
																<!--end::Table-->
															</div>
															<!--end::Tap pane-->
														</div>
													</div>
													<!--end::Body-->
												</div>
												<!--end::Tables Widget 4-->
											</div>
											<div class="tab-pane fade" id="kt_general_widget_1_5">
												<!--begin::Tables Widget 1-->
												<div class="card">
													<!--begin::Header-->
													<div class="card-header border-0 pt-5">
														<h3 class="card-title align-items-start flex-column">
															<span class="card-label fw-bold fs-3 mb-1">Tasks Overview</span>
															<span class="text-muted fw-semibold fs-7">Pending 10 tasks</span>
														</h3>
														<div class="card-toolbar">
															<!--begin::Menu-->
															<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
																<i class="ki-duotone ki-category fs-6">
																	<span class="path1"></span>
																	<span class="path2"></span>
																	<span class="path3"></span>
																	<span class="path4"></span>
																</i>
															</button>
															<!--begin::Menu 1-->
															<div class="menu menu-sub menu-sub-dropdown w-250px w-md-300px" data-kt-menu="true" id="kt_menu_66b9a49b3fa62">
																<!--begin::Header-->
																<div class="px-7 py-5">
																	<div class="fs-5 text-gray-900 fw-bold">Filter Options</div>
																</div>
																<!--end::Header-->
																<!--begin::Menu separator-->
																<div class="separator border-gray-200"></div>
																<!--end::Menu separator-->
																<!--begin::Form-->
																<div class="px-7 py-5">
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Status:</label>
																		<!--end::Label-->
																		<!--begin::Input-->
																		<div>
																			<select class="form-select form-select-solid" multiple="multiple" data-kt-select2="true" data-close-on-select="false" data-placeholder="Select option" data-dropdown-parent="#kt_menu_66b9a49b3fa62" data-allow-clear="true">
																				<option></option>
																				<option value="1">Approved</option>
																				<option value="2">Pending</option>
																				<option value="2">In Process</option>
																				<option value="2">Rejected</option>
																			</select>
																		</div>
																		<!--end::Input-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Member Type:</label>
																		<!--end::Label-->
																		<!--begin::Options-->
																		<div class="d-flex">
																			<!--begin::Options-->
																			<label class="form-check form-check-sm form-check-custom form-check-solid me-5">
																				<input class="form-check-input" type="checkbox" value="1" />
																				<span class="form-check-label">Author</span>
																			</label>
																			<!--end::Options-->
																			<!--begin::Options-->
																			<label class="form-check form-check-sm form-check-custom form-check-solid">
																				<input class="form-check-input" type="checkbox" value="2" checked="checked" />
																				<span class="form-check-label">Customer</span>
																			</label>
																			<!--end::Options-->
																		</div>
																		<!--end::Options-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Input group-->
																	<div class="mb-10">
																		<!--begin::Label-->
																		<label class="form-label fw-semibold">Notifications:</label>
																		<!--end::Label-->
																		<!--begin::Switch-->
																		<div class="form-check form-switch form-switch-sm form-check-custom form-check-solid">
																			<input class="form-check-input" type="checkbox" value="" name="notifications" checked="checked" />
																			<label class="form-check-label">Enabled</label>
																		</div>
																		<!--end::Switch-->
																	</div>
																	<!--end::Input group-->
																	<!--begin::Actions-->
																	<div class="d-flex justify-content-end">
																		<button type="reset" class="btn btn-sm btn-light btn-active-light-primary me-2" data-kt-menu-dismiss="true">Reset</button>
																		<button type="submit" class="btn btn-sm btn-primary" data-kt-menu-dismiss="true">Apply</button>
																	</div>
																	<!--end::Actions-->
																</div>
																<!--end::Form-->
															</div>
															<!--end::Menu 1-->
															<!--end::Menu-->
														</div>
													</div>
													<!--end::Header-->
													<!--begin::Body-->
													<div class="card-body py-3">
														<!--begin::Table container-->
														<div class="table-responsive">
															<!--begin::Table-->
															<table class="table align-middle gs-0 gy-5">
																<!--begin::Table head-->
																<thead>
																	<tr>
																		<th class="p-0 w-50px"></th>
																		<th class="p-0 min-w-200px"></th>
																		<th class="p-0 min-w-100px"></th>
																		<th class="p-0 min-w-40px"></th>
																	</tr>
																</thead>
																<!--end::Table head-->
																<!--begin::Table body-->
																<tbody>
																	<tr>
																		<th>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/plurk.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</th>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Top Authors</a>
																			<span class="text-muted fw-semibold d-block fs-7">Successful Fellas</span>
																		</td>
																		<td>
																			<div class="d-flex flex-column w-100 me-2">
																				<div class="d-flex flex-stack mb-2">
																					<span class="text-muted me-2 fs-7 fw-bold">70%</span>
																				</div>
																				<div class="progress h-6px w-100">
																					<div class="progress-bar bg-primary" role="progressbar" style="width: 70%" aria-valuenow="70" aria-valuemin="0" aria-valuemax="100"></div>
																				</div>
																			</div>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<th>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</th>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Popular Authors</a>
																			<span class="text-muted fw-semibold d-block fs-7">Most Successful</span>
																		</td>
																		<td>
																			<div class="d-flex flex-column w-100 me-2">
																				<div class="d-flex flex-stack mb-2">
																					<span class="text-muted me-2 fs-7 fw-bold">50%</span>
																				</div>
																				<div class="progress h-6px w-100">
																					<div class="progress-bar bg-primary" role="progressbar" style="width: 50%" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100"></div>
																				</div>
																			</div>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<th>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/vimeo.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</th>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">New Users</a>
																			<span class="text-muted fw-semibold d-block fs-7">Awesome Users</span>
																		</td>
																		<td>
																			<div class="d-flex flex-column w-100 me-2">
																				<div class="d-flex flex-stack mb-2">
																					<span class="text-muted me-2 fs-7 fw-bold">80%</span>
																				</div>
																				<div class="progress h-6px w-100">
																					<div class="progress-bar bg-primary" role="progressbar" style="width: 80%" aria-valuenow="80" aria-valuemin="0" aria-valuemax="100"></div>
																				</div>
																			</div>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<th>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</th>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Active Customers</a>
																			<span class="text-muted fw-semibold d-block fs-7">Best Customers</span>
																		</td>
																		<td>
																			<div class="d-flex flex-column w-100 me-2">
																				<div class="d-flex flex-stack mb-2">
																					<span class="text-muted me-2 fs-7 fw-bold">90%</span>
																				</div>
																				<div class="progress h-6px w-100">
																					<div class="progress-bar bg-primary" role="progressbar" style="width: 90%" aria-valuenow="90" aria-valuemin="0" aria-valuemax="100"></div>
																				</div>
																			</div>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																	<tr>
																		<th>
																			<div class="symbol symbol-50px me-2">
																				<span class="symbol-label">
																					<img src="assets/media/svg/brand-logos/kickstarter.svg" class="h-50 align-self-center" alt="" />
																				</span>
																			</div>
																		</th>
																		<td>
																			<a href="#" class="text-gray-900 fw-bold text-hover-primary mb-1 fs-6">Bestseller Theme</a>
																			<span class="text-muted fw-semibold d-block fs-7">Amazing Templates</span>
																		</td>
																		<td>
																			<div class="d-flex flex-column w-100 me-2">
																				<div class="d-flex flex-stack mb-2">
																					<span class="text-muted me-2 fs-7 fw-bold">70%</span>
																				</div>
																				<div class="progress h-6px w-100">
																					<div class="progress-bar bg-primary" role="progressbar" style="width: 70%" aria-valuenow="70" aria-valuemin="0" aria-valuemax="100"></div>
																				</div>
																			</div>
																		</td>
																		<td class="text-end">
																			<a href="#" class="btn btn-sm btn-icon btn-bg-light btn-active-color-primary">
																				<i class="ki-duotone ki-arrow-right fs-2">
																					<span class="path1"></span>
																					<span class="path2"></span>
																				</i>
																			</a>
																		</td>
																	</tr>
																</tbody>
																<!--end::Table body-->
															</table>
															<!--end::Table-->
														</div>
														<!--end::Table container-->
													</div>
													<!--end::Body-->
												</div>
												<!--endW::Tables Widget 1-->
											</div>
										</div>
										<!--end::Tab content-->
									</div>
									<!--end::General Widget 1-->
									<!--begin::Charts Widget 1-->
									<div class="card mb-10">
										<!--begin::Header-->
										<div class="card-header border-0 pt-5">
											<!--begin::Title-->
											<h3 class="card-title align-items-start flex-column">
												<span class="card-label fw-bold fs-3 mb-1">Recent Statistics</span>
												<span class="text-muted fw-semibold fs-7">More than 400 new members</span>
											</h3>
											<!--end::Title-->
											<!--begin::Toolbar-->
											<div class="card-toolbar">
												<!--begin::Menu-->
												<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
													<i class="ki-duotone ki-category fs-6">
														<span class="path1"></span>
														<span class="path2"></span>
														<span class="path3"></span>
														<span class="path4"></span>
													</i>
												</button>
												<!--begin::Menu 1-->
												<div class="menu menu-sub menu-sub-dropdown w-250px w-md-300px" data-kt-menu="true" id="kt_menu_66b9a49b3fb1e">
													<!--begin::Header-->
													<div class="px-7 py-5">
														<div class="fs-5 text-gray-900 fw-bold">Filter Options</div>
													</div>
													<!--end::Header-->
													<!--begin::Menu separator-->
													<div class="separator border-gray-200"></div>
													<!--end::Menu separator-->
													<!--begin::Form-->
													<div class="px-7 py-5">
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Status:</label>
															<!--end::Label-->
															<!--begin::Input-->
															<div>
																<select class="form-select form-select-solid" multiple="multiple" data-kt-select2="true" data-close-on-select="false" data-placeholder="Select option" data-dropdown-parent="#kt_menu_66b9a49b3fb1e" data-allow-clear="true">
																	<option></option>
																	<option value="1">Approved</option>
																	<option value="2">Pending</option>
																	<option value="2">In Process</option>
																	<option value="2">Rejected</option>
																</select>
															</div>
															<!--end::Input-->
														</div>
														<!--end::Input group-->
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Member Type:</label>
															<!--end::Label-->
															<!--begin::Options-->
															<div class="d-flex">
																<!--begin::Options-->
																<label class="form-check form-check-sm form-check-custom form-check-solid me-5">
																	<input class="form-check-input" type="checkbox" value="1" />
																	<span class="form-check-label">Author</span>
																</label>
																<!--end::Options-->
																<!--begin::Options-->
																<label class="form-check form-check-sm form-check-custom form-check-solid">
																	<input class="form-check-input" type="checkbox" value="2" checked="checked" />
																	<span class="form-check-label">Customer</span>
																</label>
																<!--end::Options-->
															</div>
															<!--end::Options-->
														</div>
														<!--end::Input group-->
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Notifications:</label>
															<!--end::Label-->
															<!--begin::Switch-->
															<div class="form-check form-switch form-switch-sm form-check-custom form-check-solid">
																<input class="form-check-input" type="checkbox" value="" name="notifications" checked="checked" />
																<label class="form-check-label">Enabled</label>
															</div>
															<!--end::Switch-->
														</div>
														<!--end::Input group-->
														<!--begin::Actions-->
														<div class="d-flex justify-content-end">
															<button type="reset" class="btn btn-sm btn-light btn-active-light-primary me-2" data-kt-menu-dismiss="true">Reset</button>
															<button type="submit" class="btn btn-sm btn-primary" data-kt-menu-dismiss="true">Apply</button>
														</div>
														<!--end::Actions-->
													</div>
													<!--end::Form-->
												</div>
												<!--end::Menu 1-->
												<!--end::Menu-->
											</div>
											<!--end::Toolbar-->
										</div>
										<!--end::Header-->
										<!--begin::Body-->
										<div class="card-body">
											<!--begin::Chart-->
											<div id="kt_charts_widget_1_chart" style="height: 350px"></div>
											<!--end::Chart-->
										</div>
										<!--end::Body-->
									</div>
									<!--end::Charts Widget 1-->
								</div>
								<!--end::Col-->
								<!--begin::Col-->
								<div class="col-xl-4">
									<!--begin::List Widget 5-->
									<div class="card mb-10">
										<!--begin::Header-->
										<div class="card-header align-items-center border-0 mt-4">
											<h3 class="card-title align-items-start flex-column">
												<span class="fw-bold mb-2 text-gray-900">Activities</span>
												<span class="text-muted fw-semibold fs-7">890,344 Sales</span>
											</h3>
											<div class="card-toolbar">
												<!--begin::Menu-->
												<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
													<i class="ki-duotone ki-category fs-6">
														<span class="path1"></span>
														<span class="path2"></span>
														<span class="path3"></span>
														<span class="path4"></span>
													</i>
												</button>
												<!--begin::Menu 1-->
												<div class="menu menu-sub menu-sub-dropdown w-250px w-md-300px" data-kt-menu="true" id="kt_menu_66b9a49b3fb7d">
													<!--begin::Header-->
													<div class="px-7 py-5">
														<div class="fs-5 text-gray-900 fw-bold">Filter Options</div>
													</div>
													<!--end::Header-->
													<!--begin::Menu separator-->
													<div class="separator border-gray-200"></div>
													<!--end::Menu separator-->
													<!--begin::Form-->
													<div class="px-7 py-5">
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Status:</label>
															<!--end::Label-->
															<!--begin::Input-->
															<div>
																<select class="form-select form-select-solid" multiple="multiple" data-kt-select2="true" data-close-on-select="false" data-placeholder="Select option" data-dropdown-parent="#kt_menu_66b9a49b3fb7d" data-allow-clear="true">
																	<option></option>
																	<option value="1">Approved</option>
																	<option value="2">Pending</option>
																	<option value="2">In Process</option>
																	<option value="2">Rejected</option>
																</select>
															</div>
															<!--end::Input-->
														</div>
														<!--end::Input group-->
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Member Type:</label>
															<!--end::Label-->
															<!--begin::Options-->
															<div class="d-flex">
																<!--begin::Options-->
																<label class="form-check form-check-sm form-check-custom form-check-solid me-5">
																	<input class="form-check-input" type="checkbox" value="1" />
																	<span class="form-check-label">Author</span>
																</label>
																<!--end::Options-->
																<!--begin::Options-->
																<label class="form-check form-check-sm form-check-custom form-check-solid">
																	<input class="form-check-input" type="checkbox" value="2" checked="checked" />
																	<span class="form-check-label">Customer</span>
																</label>
																<!--end::Options-->
															</div>
															<!--end::Options-->
														</div>
														<!--end::Input group-->
														<!--begin::Input group-->
														<div class="mb-10">
															<!--begin::Label-->
															<label class="form-label fw-semibold">Notifications:</label>
															<!--end::Label-->
															<!--begin::Switch-->
															<div class="form-check form-switch form-switch-sm form-check-custom form-check-solid">
																<input class="form-check-input" type="checkbox" value="" name="notifications" checked="checked" />
																<label class="form-check-label">Enabled</label>
															</div>
															<!--end::Switch-->
														</div>
														<!--end::Input group-->
														<!--begin::Actions-->
														<div class="d-flex justify-content-end">
															<button type="reset" class="btn btn-sm btn-light btn-active-light-primary me-2" data-kt-menu-dismiss="true">Reset</button>
															<button type="submit" class="btn btn-sm btn-primary" data-kt-menu-dismiss="true">Apply</button>
														</div>
														<!--end::Actions-->
													</div>
													<!--end::Form-->
												</div>
												<!--end::Menu 1-->
												<!--end::Menu-->
											</div>
										</div>
										<!--end::Header-->
										<!--begin::Body-->
										<div class="card-body pt-5">
											<!--begin::Timeline-->
											<div class="timeline-label">
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">08:42</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-warning fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Text-->
													<div class="fw-mormal timeline-content text-muted ps-3">Outlines keep you honest. And keep structure</div>
													<!--end::Text-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">10:00</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-success fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Content-->
													<div class="timeline-content d-flex">
														<span class="fw-bold text-gray-800 ps-3">AEOL meeting</span>
													</div>
													<!--end::Content-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">14:37</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-danger fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Desc-->
													<div class="timeline-content fw-bold text-gray-800 ps-3">Make deposit 
													<a href="#" class="text-primary">USD 700</a>. to ESL</div>
													<!--end::Desc-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">16:50</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-primary fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Text-->
													<div class="timeline-content fw-mormal text-muted ps-3">Indulging in poorly driving and keep structure keep great</div>
													<!--end::Text-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">21:03</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-danger fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Desc-->
													<div class="timeline-content fw-semibold text-gray-800 ps-3">New order placed 
													<a href="#" class="text-primary">#XF-2356</a>.</div>
													<!--end::Desc-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">16:50</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-primary fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Text-->
													<div class="timeline-content fw-mormal text-muted ps-3">Indulging in poorly driving and keep structure keep great</div>
													<!--end::Text-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">21:03</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-danger fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Desc-->
													<div class="timeline-content fw-semibold text-gray-800 ps-3">New order placed 
													<a href="#" class="text-primary">#XF-2356</a>.</div>
													<!--end::Desc-->
												</div>
												<!--end::Item-->
												<!--begin::Item-->
												<div class="timeline-item">
													<!--begin::Label-->
													<div class="timeline-label fw-bold text-gray-800 fs-6">10:30</div>
													<!--end::Label-->
													<!--begin::Badge-->
													<div class="timeline-badge">
														<i class="fa fa-genderless text-success fs-1"></i>
													</div>
													<!--end::Badge-->
													<!--begin::Text-->
													<div class="timeline-content fw-mormal text-muted ps-3">Finance KPI Mobile app launch preparion meeting</div>
													<!--end::Text-->
												</div>
												<!--end::Item-->
											</div>
											<!--end::Timeline-->
										</div>
										<!--end: Card Body-->
									</div>
									<!--end: List Widget 5-->
									<!--begin::List Widget 4-->
									<div class="card">
										<!--begin::Header-->
										<div class="card-header border-0 pt-5">
											<h3 class="card-title align-items-start flex-column">
												<span class="card-label fw-bold text-gray-900">Trends</span>
												<span class="text-muted mt-1 fw-semibold fs-7">Latest tech trends</span>
											</h3>
											<div class="card-toolbar">
												<!--begin::Menu-->
												<button type="button" class="btn btn-sm btn-icon btn-color-primary btn-active-light-primary" data-kt-menu-trigger="click" data-kt-menu-placement="bottom-end">
													<i class="ki-duotone ki-category fs-6">
														<span class="path1"></span>
														<span class="path2"></span>
														<span class="path3"></span>
														<span class="path4"></span>
													</i>
												</button>
												<!--begin::Menu 3-->
												<div class="menu menu-sub menu-sub-dropdown menu-column menu-rounded menu-gray-800 menu-state-bg-light-primary fw-semibold w-200px py-3" data-kt-menu="true">
													<!--begin::Heading-->
													<div class="menu-item px-3">
														<div class="menu-content text-muted pb-2 px-3 fs-7 text-uppercase">Payments</div>
													</div>
													<!--end::Heading-->
													<!--begin::Menu item-->
													<div class="menu-item px-3">
														<a href="#" class="menu-link px-3">Create Invoice</a>
													</div>
													<!--end::Menu item-->
													<!--begin::Menu item-->
													<div class="menu-item px-3">
														<a href="#" class="menu-link flex-stack px-3">Create Payment 
														<span class="ms-2" data-bs-toggle="tooltip" title="Specify a target name for future usage and reference">
															<i class="ki-duotone ki-information fs-6">
																<span class="path1"></span>
																<span class="path2"></span>
																<span class="path3"></span>
															</i>
														</span></a>
													</div>
													<!--end::Menu item-->
													<!--begin::Menu item-->
													<div class="menu-item px-3">
														<a href="#" class="menu-link px-3">Generate Bill</a>
													</div>
													<!--end::Menu item-->
													<!--begin::Menu item-->
													<div class="menu-item px-3" data-kt-menu-trigger="hover" data-kt-menu-placement="right-end">
														<a href="#" class="menu-link px-3">
															<span class="menu-title">Subscription</span>
															<span class="menu-arrow"></span>
														</a>
														<!--begin::Menu sub-->
														<div class="menu-sub menu-sub-dropdown w-175px py-4">
															<!--begin::Menu item-->
															<div class="menu-item px-3">
																<a href="#" class="menu-link px-3">Plans</a>
															</div>
															<!--end::Menu item-->
															<!--begin::Menu item-->
															<div class="menu-item px-3">
																<a href="#" class="menu-link px-3">Billing</a>
															</div>
															<!--end::Menu item-->
															<!--begin::Menu item-->
															<div class="menu-item px-3">
																<a href="#" class="menu-link px-3">Statements</a>
															</div>
															<!--end::Menu item-->
															<!--begin::Menu separator-->
															<div class="separator my-2"></div>
															<!--end::Menu separator-->
															<!--begin::Menu item-->
															<div class="menu-item px-3">
																<div class="menu-content px-3">
																	<!--begin::Switch-->
																	<label class="form-check form-switch form-check-custom form-check-solid">
																		<!--begin::Input-->
																		<input class="form-check-input w-30px h-20px" type="checkbox" value="1" checked="checked" name="notifications" />
																		<!--end::Input-->
																		<!--end::Label-->
																		<span class="form-check-label text-muted fs-6">Recuring</span>
																		<!--end::Label-->
																	</label>
																	<!--end::Switch-->
																</div>
															</div>
															<!--end::Menu item-->
														</div>
														<!--end::Menu sub-->
													</div>
													<!--end::Menu item-->
													<!--begin::Menu item-->
													<div class="menu-item px-3 my-1">
														<a href="#" class="menu-link px-3">Settings</a>
													</div>
													<!--end::Menu item-->
												</div>
												<!--end::Menu 3-->
												<!--end::Menu-->
											</div>
										</div>
										<!--end::Header-->
										<!--begin::Body-->
										<div class="card-body pt-5">
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center mb-7">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/plurk.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">Top Authors</a>
														<span class="text-muted fw-semibold d-block fs-7">Mark, Rowling, Esther</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+82$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center mb-7">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/telegram.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">Popular Authors</a>
														<span class="text-muted fw-semibold d-block fs-7">Randy, Steve, Mike</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+280$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center mb-7">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/vimeo.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">New Users</a>
														<span class="text-muted fw-semibold d-block fs-7">John, Pat, Jimmy</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+4500$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center mb-7">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/bebo.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">Active Customers</a>
														<span class="text-muted fw-semibold d-block fs-7">Mark, Rowling, Esther</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+686$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center mb-7">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/kickstarter.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">Bestseller Theme</a>
														<span class="text-muted fw-semibold d-block fs-7">Disco, Retro, Sports</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+726$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
											<!--begin::Item-->
											<div class="d-flex align-items-sm-center">
												<!--begin::Symbol-->
												<div class="symbol symbol-50px me-5">
													<span class="symbol-label">
														<img src="assets/media/svg/brand-logos/fox-hub.svg" class="h-50 align-self-center" alt="" />
													</span>
												</div>
												<!--end::Symbol-->
												<!--begin::Section-->
												<div class="d-flex align-items-center flex-row-fluid flex-wrap">
													<div class="flex-grow-1 me-2">
														<a href="#" class="text-gray-800 text-hover-primary fs-6 fw-bold">Fox Broker App</a>
														<span class="text-muted fw-semibold d-block fs-7">Finance, Corporate, Apps</span>
													</div>
													<span class="badge badge-light fw-bold my-2">+145$</span>
												</div>
												<!--end::Section-->
											</div>
											<!--end::Item-->
										</div>
										<!--end::Body-->
									</div>
									<!--end::List Widget 4-->
								</div>
								<!--end::Col-->
							</div>
							<!--end::Row-->
    </div>

@endsection

@push('scripts')
    <script>
        $(document).ready(function() {
         
        });
    </script>
@endpush
