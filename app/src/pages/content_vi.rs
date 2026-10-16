//! The footer pages in Vietnamese: generated from today's site's
//! `web/src/locales/vi.json` `pages.*` (verbatim; `{{publisher}}`,
//! `{{contact}}` and `{{date}}` resolved; the Legal page and the two
//! pointers to it dropped — task 001, fix round 4; AI mentions removed —
//! task 004). Data, not code: regenerate rather than edit by hand.

use super::{AppBlock, Cta, Href, InstagramBlock, PageText, Pages, Section, Table, ToolBlock};

/// Every footer page, in this language.
pub(super) const PAGES: Pages = Pages {
    acknowledgements: PageText {
        title: "Tri ân",
        description: "Ghi công cho các bài kinh, bản dịch và từ điển mà PaliCanon tìm kiếm, cùng giấy phép của chúng.",
        intro: &[
            "PaliCanon là một công cụ tìm kiếm, không phải một nguồn kinh văn. Mọi thứ bạn tìm thấy ở đây đều do người khác tạo ra và rộng lòng trao tặng; trang này ghi lại đó là ai, và theo giấy phép nào.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "Các bài kinh",
                p: &[
                    "Bản dịch tiếng Anh là của **Tỳ-kheo Sujato**, người đã dịch bốn bộ Nikāya chính cùng phần lớn Tiểu bộ (Khuddaka) và hiến tặng toàn bộ cho công chúng. Văn bản Pāli là ấn bản **Mahāsaṅgīti Tipiṭaka** do Dhamma Society biên soạn. Cả hai được [SuttaCentral](https://suttacentral.net) xuất bản theo tuyên bố Creative Commons CC0, cùng với tên kinh, phần tóm lược và hệ thống ký hiệu mà PaliCanon sử dụng. Mỗi kết quả đều dẫn về SuttaCentral; xin hãy đọc trọn bài kinh ở đó, và cân nhắc [ủng hộ họ](https://suttacentral.net/donations).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Bản dịch tiếng Pháp và tiếng Việt",
                p: &[
                    "Khi PaliCanon được dùng bằng tiếng Pháp hoặc tiếng Việt, tên kinh được lấy từ các bản dịch do [SuttaCentral](https://suttacentral.net) công bố (mỗi bản dịch thuộc về dịch giả của nó), nếu bài kinh có bản dịch đó; việc tìm kiếm cũng dựa trên các bản dịch này, nên câu hỏi bằng tiếng Pháp hay tiếng Việt được trả lời từ chính các bản ấy. Các bản tiếng Pháp (1.184 bài kinh) do **Môhan Wijayaratna**, **Rémy Zins**, **Claude & Chandhana Le Ninan**, **Christelle Ah-Mouck**, **Marion Ah-Mouck**, **Fernand Hû**, **U Ko Lay**, **U Tin U (Myaung)**, **Florent Robert**, **Indavati**, **Régis Xhardé** và nhóm **buddha-vacana.org**, **Canonpali.org** dịch. Các bản tiếng Việt (4.240 bài kinh) do **Thích Minh Châu**, **Bhikkhu Indacanda** và **Phan Tuấn Anh** dịch. Tất cả được thu thập từ [SuttaCentral](https://suttacentral.net) và giữ nguyên văn; các bản dịch thuộc về dịch giả của chúng.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Từ điển",
                p: &[
                    "Nghĩa của các từ Pāli lấy từ **Digital Pāḷi Dictionary** của Tỳ-kheo Bodhirasa, [digitalpalidictionary.github.io](https://digitalpalidictionary.github.io), theo giấy phép [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/). PaliCanon phi thương mại và giữ nguyên các mục từ; mọi việc sử dụng dữ liệu từ điển về sau đều phải tôn trọng giấy phép đó.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "Với lòng biết ơn",
                p: &[
                    "Xin tri ân Tỳ-kheo Sujato, đã dịch Tạng Kinh (Sutta Piṭaka) và trao tặng cho mọi người; Tỳ-kheo Bodhirasa, vì bộ từ điển; nhóm SuttaCentral, đã xuất bản và gìn giữ kinh văn; và dự án Mahāsaṅgīti của Dhamma Society, vì ấn bản Pāli.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    terms: PageText {
        title: "Điều khoản sử dụng",
        description: "Các điều khoản áp dụng khi sử dụng PaliCanon.",
        intro: &[
            "Bản tiếng Việt này chỉ nhằm giúp bạn dễ đọc; nếu có khác biệt, bản tiếng Anh được ưu tiên áp dụng.",
            "Các điều khoản này điều chỉnh việc bạn sử dụng PaliCanon (“Dịch vụ”), do Thibault Rivière phát hành (“Bên phát hành”). Khi sử dụng Dịch vụ, bạn chấp nhận các điều khoản này.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. Dịch vụ",
                p: &[
                    "Dịch vụ cho phép bạn tìm trong các bài kinh của Tạng Kinh (Sutta Piṭaka) theo đúng từ ngữ và theo ý nghĩa, đọc những đoạn tìm được kèm liên kết đến nguồn, và tra nghĩa từ Pāli. Dịch vụ được cung cấp miễn phí và không cần đăng ký.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "2. Chấp nhận",
                p: &[
                    "Sử dụng Dịch vụ nghĩa là chấp nhận các điều khoản này. Bạn có thể ngừng sử dụng Dịch vụ bất cứ lúc nào; dữ liệu lưu trong trình duyệt là của bạn và bạn có thể tự xóa (xem [Quyền riêng tư](/privacy)).",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. Kinh văn và sở hữu trí tuệ",
                p: &[
                    "Các bài kinh thuộc phạm vi công cộng và từ điển theo giấy phép CC BY-NC 4.0, như được nêu chi tiết trong [Tri ân](/acknowledgements). Bản thân Dịch vụ — công cụ tìm kiếm, giao diện, chỉ mục và tên gọi — là tài sản của Bên phát hành. Bạn được dùng Dịch vụ để tra cứu cá nhân, phi thương mại. Việc trích xuất hàng loạt kết quả bằng phương tiện tự động, sao chép Dịch vụ, hoặc sử dụng API của Dịch vụ bên ngoài trang web công khai đều cần có sự cho phép trước bằng văn bản của Bên phát hành.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Tính sẵn có và trách nhiệm",
                p: &[
                    "Dịch vụ được cung cấp “nguyên trạng”, miễn phí. Bên phát hành cố gắng hết sức để Dịch vụ chính xác và luôn sẵn sàng nhưng không đưa ra bất kỳ bảo đảm nào, và có thể thay đổi, tạm ngừng hoặc chấm dứt Dịch vụ bất cứ lúc nào. Trong phạm vi pháp luật cho phép, Bên phát hành không chịu trách nhiệm về bất kỳ tổn thất nào phát sinh từ việc sử dụng Dịch vụ, từ việc Dịch vụ không khả dụng, hoặc từ nội dung của các trang bên thứ ba mà Dịch vụ liên kết đến. Không điều nào ở đây giới hạn trách nhiệm mà pháp luật không cho phép giới hạn.",
                    "Bạn đồng ý không làm gián đoạn Dịch vụ, không gửi lưu lượng lạm dụng hoặc tự động, và không tìm cách truy cập các hệ thống khác ngoài trang web công khai. Bên phát hành có thể hạn chế truy cập từ bất kỳ nguồn nào vi phạm điều này.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "5. Luật áp dụng và thay đổi",
                p: &[
                    "Các điều khoản này chịu sự điều chỉnh của pháp luật nơi Bên phát hành cư trú, không ảnh hưởng đến các quy định bắt buộc về bảo vệ người tiêu dùng tại quốc gia của bạn. Bên phát hành có thể cập nhật các điều khoản; ngày ghi bên dưới là ngày của phiên bản hiện hành, và việc tiếp tục sử dụng sau khi có thay đổi được xem là chấp nhận thay đổi đó.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    privacy: PageText {
        title: "Quyền riêng tư",
        description: "Thông báo về quyền riêng tư của PaliCanon theo GDPR: dữ liệu nào được xử lý, vì sao, và trong bao lâu.",
        intro: &[
            "Bản tiếng Việt này chỉ nhằm giúp bạn dễ đọc; nếu có khác biệt, bản tiếng Anh được ưu tiên áp dụng.",
            "Đây là thông báo về quyền riêng tư theo yêu cầu của Quy định chung về bảo vệ dữ liệu của Liên minh châu Âu (GDPR). Bên kiểm soát dữ liệu là Thibault Rivière, liên hệ riviere.thibault@icloud.com. PaliCanon không có tài khoản người dùng, không đặt cookie, và không dùng quảng cáo hay công cụ theo dõi của bên thứ ba.",
        ],
        tools: &[],
        app: None,
        instagram: None,
        sections: &[
            Section {
                h: "1. Dữ liệu nào được xử lý, vì sao, và trên cơ sở nào",
                p: &[
                    "Xin đừng gõ thông tin cá nhân — của bạn hay của bất kỳ ai — vào ô tìm kiếm: các truy vấn được lưu lại, và chúng ẩn danh chỉ vì không có gì định danh được gắn kèm.",
                ],
                list: &[],
                table: Some(Table {
                    head: &[
                        "Dữ liệu",
                        "Mục đích",
                        "Cơ sở pháp lý (GDPR điều 6)",
                        "Thời gian lưu",
                    ],
                    rows: &[
                        &[
                            "Câu truy vấn tìm kiếm: nội dung đã gõ, quốc gia gửi truy vấn, thời điểm (làm tròn đến giờ), và kết quả của lần tìm (số kết quả, có sửa chính tả hay không, độ trễ). Không kèm địa chỉ IP hay bất kỳ mã định danh nào.",
                            "Cải thiện việc tìm kiếm: biết người đọc tìm gì, điều gì không tìm ra kết quả, những cách viết và thuật ngữ Pāli nào được dùng.",
                            "Lợi ích chính đáng (6.1.f): vận hành và cải thiện một dịch vụ công cộng miễn phí, với dữ liệu không thể gắn với một cá nhân.",
                            "Tối đa 24 tháng; số liệu thống kê tổng hợp được giữ vô thời hạn.",
                        ],
                        &[
                            "Nhật ký kết nối do các nhà cung cấp dịch vụ lưu trữ ghi lại: địa chỉ IP, thời điểm, URL được yêu cầu, user agent.",
                            "Bảo mật, ngăn chặn lạm dụng, duy trì hoạt động của Dịch vụ.",
                            "Lợi ích chính đáng (6.1.f).",
                            "Do các nhà cung cấp lưu theo chính sách riêng của họ; thường là 30 ngày hoặc ít hơn.",
                        ],
                    ],
                }),
                cta: None,
            },
            Section {
                h: "2. Bên nhận dữ liệu và việc chuyển dữ liệu",
                p: &[
                    "Trang web được phục vụ bởi Cloudflare, Inc. (Hoa Kỳ) và API tìm kiếm bởi Google Cloud (Google Ireland Limited; máy chủ có thể đặt tại EU hoặc Hoa Kỳ). Cả hai đóng vai trò bên xử lý dữ liệu theo các điều khoản xử lý dữ liệu tiêu chuẩn của họ, bao gồm các điều khoản hợp đồng mẫu của Ủy ban châu Âu cho việc chuyển dữ liệu ra ngoài EU. Không dữ liệu nào được bán hay chia sẻ với bất kỳ ai khác.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "3. Những gì ở lại trong trình duyệt của bạn",
                p: &[
                    "Giao diện và ngôn ngữ của bạn được trình duyệt lưu lại (localStorage) và không bao giờ được gửi đến Dịch vụ. Việc lưu trữ này là thật sự cần thiết để cung cấp các tính năng bạn yêu cầu, nên không cần đến thông báo xin đồng ý. Bạn có thể xóa bất cứ lúc nào bằng cách xóa dữ liệu của trang web trong trình duyệt.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
            Section {
                h: "4. Quyền của bạn",
                p: &[
                    "Bạn có các quyền truy cập, chỉnh sửa, xóa, hạn chế xử lý, di chuyển dữ liệu và phản đối theo GDPR, cùng quyền khiếu nại với cơ quan giám sát (tại Pháp là CNIL). Hãy viết thư đến riviere.thibault@icloud.com. Lưu ý rằng các truy vấn tìm kiếm được lưu mà không có mã định danh nào: Dịch vụ không thể tìm ra “các truy vấn của bạn”, và vì vậy không thể trả lại hay xóa riêng từng truy vấn — đó cũng chính là điều giữ cho chúng ẩn danh.",
                ],
                list: &[],
                table: None,
                cta: None,
            },
        ],
    },
    support: PageText {
        title: "Ủng hộ",
        description: "Những gì cần để duy trì PaliCanon và ứng dụng Tipiṭaka, và cách bạn có thể giúp giữ chúng miễn phí.",
        intro: &[
            "PaliCanon và ứng dụng Tipiṭaka cho iPhone đều miễn phí: không quảng cáo, không tài khoản, không có gì của bạn bị đem bán. Một người duy nhất xây dựng và duy trì chúng.",
        ],
        tools: &[
            ToolBlock {
                id: "search",
                h: "Tìm kiếm",
                tagline: "Tìm trong toàn bộ Kinh tạng Pāli theo những từ bạn nhớ hoặc theo ý bạn muốn nói: bằng tiếng Anh, tiếng Pháp và tiếng Việt, kèm đoạn kinh và đường dẫn để đọc.",
                cta: "Tìm trong Kinh tạng",
                href: Href::Path("/"),
                shots: &[
                    "/support/search-home.jpg",
                    "/support/search-results.jpg",
                    "/support/search-words.jpg",
                    "/support/search-french.jpg",
                    "/support/search-dark.jpg",
                ],
                alts: &[
                    "Trang chủ của công cụ tìm kiếm",
                    "Kết quả cho một câu hỏi",
                    "Kết quả theo từ chính xác",
                    "Một lượt tìm bằng tiếng Pháp",
                    "Giao diện tối",
                ],
            },
            ToolBlock {
                id: "learn",
                h: "Học tiếng Pāli",
                tagline: "Học ngôn ngữ của kinh điển từng từ một, với những từ lấy từ chính Kinh tạng và được ôn lại theo nhịp của trí nhớ bạn.",
                cta: "Bắt đầu học tiếng Pāli",
                href: Href::Url("https://learn.palicanon.app"),
                shots: &[
                    "/support/learn-1.jpg",
                    "/support/learn-2.jpg",
                    "/support/learn-3.jpg",
                    "/support/learn-4.jpg",
                    "/support/learn-5.jpg",
                ],
                alts: &[
                    "Trang chủ của phần học tiếng Pāli",
                    "Thêm từ mới vào bộ thẻ",
                    "Một thẻ hỏi nghĩa của một từ Pāli",
                    "Đáp án, kèm một dòng kinh",
                    "Một dòng kinh có từ cần tìm",
                ],
            },
            ToolBlock {
                id: "map",
                h: "Bản đồ",
                tagline: "Xem Kinh tạng Pāli như một bản đồ ba chiều: các bài kinh nối với nhau theo bài tương đương và theo nội dung.",
                cta: "Mở bản đồ",
                href: Href::Url("https://map.palicanon.app"),
                shots: &[
                    "/support/map-parallels.jpg",
                    "/support/map-meaning.jpg",
                    "/support/map-cluster.jpg",
                    "/support/map-phone.jpg",
                ],
                alts: &[
                    "Cách nhìn theo kinh tương đương, theo SuttaCentral",
                    "Cách nhìn theo ý nghĩa, bằng embeddings",
                    "Một bài kinh và những bài gần nhất",
                    "Các bộ kinh và ngưỡng tương đồng",
                ],
            },
        ],
        app: Some(AppBlock {
            h: "Ứng dụng Tipiṭaka",
            tagline: "Trọn bộ Tam Tạng Pāli trên iPhone của bạn, dùng được khi ngoại tuyến: Pāli và bản dịch song song, từ điển cho từng từ, lộ trình học, ghi chú — bằng tiếng Anh, tiếng Pháp và tiếng Việt.",
            cta: "Tải ứng dụng cho iPhone",
            alts: &[
                "Màn hình chính của ứng dụng Tipiṭaka",
                "Đọc một bài kinh, Pāli và bản dịch song song",
                "Tìm kiếm trong Tam Tạng theo từ hoặc theo ý nghĩa",
                "Sơ đồ khái niệm liên kết các đoạn kinh liên quan",
                "Ghi chú trên một đoạn kinh",
            ],
        }),
        instagram: Some(InstagramBlock {
            h: "Trên Instagram",
            tagline: "Mỗi ngày một bài kinh, kèm một suy ngẫm ngắn.",
            cta: "Theo dõi trên Instagram",
        }),
        sections: &[
            Section {
                h: "Chi phí duy trì",
                p: &[],
                list: &[
                    "Các máy chủ chạy công cụ tìm kiếm và chỉ mục theo ý nghĩa — dịch vụ lưu trữ đám mây.",
                    "Tên miền và hạ tầng mạng phía trước trang web.",
                    "Khoản phí thành viên nhà phát triển Apple để ứng dụng có mặt trên App Store.",
                    "Năng lực tính toán để xây dựng và xây dựng lại các chỉ mục cùng các bản dịch.",
                    "Thời gian: xây dựng, sửa lỗi, thêm ngôn ngữ mới.",
                ],
                table: None,
                cta: None,
            },
            Section {
                h: "Cách giúp đỡ",
                p: &[
                    "Nếu hoan hỷ, một khoản cúng dường trên Ko-fi — một lần hoặc hằng tháng — sẽ được dùng trực tiếp cho các chi phí trên.",
                ],
                list: &[],
                table: None,
                cta: Some(Cta {
                    label: "Ủng hộ trên Ko-fi",
                    href: "https://ko-fi.com/tipitaka",
                }),
            },
            Section {
                h: "Những cách khác, không tốn gì cả",
                p: &[
                    "Pháp (Dhamma) vốn luôn được trao tặng tự do; điều này giúp sự cúng dường ấy tiếp tục.",
                ],
                list: &[
                    "Chia sẻ một bài kinh bạn đã tìm thấy với ai đó.",
                    "Đánh giá ứng dụng trên [App Store](https://apps.apple.com/app/apple-store/id6760995639?pt=128678815&ct=PaliWebSupport&mt=8).",
                    "Báo lỗi: viết thư đến riviere.thibault@icloud.com.",
                ],
                table: None,
                cta: None,
            },
        ],
    },
    updated: "Cập nhật lần cuối: 2026-10-01.",
};
