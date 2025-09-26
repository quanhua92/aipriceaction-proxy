use crate::ask_ai::types::AskAITemplate;

pub fn get_single_ticker_templates_vn() -> Vec<AskAITemplate> {
    vec![
        // TOP 5 IMMEDIATE ACTION PROMPTS
        AskAITemplate {
            id: "should-hold-sell-buy-more".to_string(),
            title: "🎯 Tôi nên giữ, bán hay mua thêm NGAY BÂY GIỜ?".to_string(),
            prompt: "Dựa trên TẤT CẢ dữ liệu có sẵn (phân tích kỹ thuật, VPA, dòng tiền, bài viết và bối cảnh thị trường), tôi nên giữ, bán hay mua thêm mã này? Phân tích: (1) Mô hình dòng tiền thông minh so với các cổ phiếu dẫn đầu thị trường, (2) Vị trí luân chuyển ngành hiện tại, (3) Mức vào/ra kỹ thuật với giá cụ thể, (4) Tỷ lệ rủi ro/lợi nhuận trong 2-4 tuần tới, (5) Quy mô vị thế và thời điểm tối ưu. Đưa ra khuyến nghị có thể hành động với mục tiêu giá và mức cắt lỗ cụ thể.".to_string(),
        },
        AskAITemplate {
            id: "panic-decision".to_string(),
            title: "🚨 Quyết định khẩn cấp - Phản ứng với sụp đổ hoặc tăng vọt".to_string(),
            prompt: "Nếu mã này đột nhiên sụp đổ (bán tháo hoảng loạn) hoặc tăng vọt (FOMO), tôi nên làm gì NGAY LẬP TỨC? Phân tích các mô hình dòng tiền trong lúc hoảng loạn - các tổ chức có đang tích lũy hay phân phối? So sánh với hành vi của các cổ phiếu dẫn đầu thị trường và ngành. Đưa ra quyết định tức thì: (1) Mua bằng mọi giá với lý do, (2) Bán gấp để cắt lỗ, hoặc (3) Giữ vững và chờ đợi. Bao gồm mức giá chính xác và quy mô vị thế cho hành động khẩn cấp.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-analysis".to_string(),
            title: "💰 Dòng tiền vs Tiền thông minh".to_string(),
            prompt: "Phân tích các mô hình dòng tiền của mã này so với hành vi của tiền thông minh. So sánh phần trăm dòng tiền của nó với các cổ phiếu dẫn đầu thị trường và ngành. Xác định: (1) Tiền thông minh có đang chảy VÀO hay RA? (2) Tín hiệu tích lũy vs phân phối của tiền thông minh, (3) Hoạt động của tiền thông minh so với xu hướng thị trường rộng lớn, (4) Mức độ tin tưởng của tiền thông minh dựa trên xếp hạng dòng tiền, (5) Có nên theo hoặc đi ngược lại vị thế của tiền thông minh để tối đa hóa lợi nhuận.".to_string(),
        },
        AskAITemplate {
            id: "optimal-position-action".to_string(),
            title: "⚡ Kế hoạch hành động tối ưu cho vị thế hiện tại".to_string(),
            prompt: "Với vị thế hiện tại của tôi trong mã này, hành động tối ưu NGAY BÂY GIỜ là gì? Phân tích: (1) Tôi có nên tăng, giảm hay giữ vị thế ổn định? (2) Giá vào/ra chính xác cho bất kỳ thay đổi nào, (3) Điều chỉnh quy mô vị thế với phần trăm, (4) Thời gian thực hiện tối ưu (hôm nay, tuần này, chờ tín hiệu), (5) Quản lý rủi ro cho các biến động bất ngờ. Cung cấp kế hoạch hành động từng bước để tối đa hóa lợi nhuận.".to_string(),
        },
        AskAITemplate {
            id: "profit-maximization".to_string(),
            title: "🚀 Chiến lược lợi nhuận tối đa - Setup tốt nhất có sẵn".to_string(),
            prompt: "Dựa trên TẤT CẢ dữ liệu có sẵn (biểu đồ, VPA, dòng tiền, bài viết, cơ bản), chiến lược XÁC SUẤT CAO NHẤT để tối đa hóa lợi nhuận từ mã này là gì? Phân tích: (1) Quy mô vị thế tối ưu sử dụng tiêu chí Kelly, (2) Thời điểm và mức giá vào tốt nhất, (3) Chiến lược chốt lời theo tầng, (4) Quản lý rủi ro với cắt lỗ di động, (5) Thời gian cho lợi nhuận tối đa (ngày, tuần, tháng). Tập trung vào cơ hội lợi nhuận tốt nhất tuyệt đối có sẵn.".to_string(),
        },

        // PHÂN TÍCH KỸ THUẬT & THỜI ĐIỂM
        AskAITemplate {
            id: "technical-analysis-complete".to_string(),
            title: "📊 Phân tích kỹ thuật đầy đủ & Mô hình biểu đồ".to_string(),
            prompt: "Cung cấp phân tích kỹ thuật toàn diện kết hợp TẤT CẢ chỉ báo: (1) Mô hình biểu đồ (tam giác, cờ, đầu vai, v.v.) với mục tiêu breakout, (2) Mức hỗ trợ/kháng cự với giá chính xác, (3) Phân tích xu hướng với đường MA, (4) Phân tích khối lượng và tín hiệu VPA, (5) Chỉ báo momentum và phân kỳ, (6) Fibonacci retracements và extensions. Bao gồm điểm vào/ra cụ thể và mục tiêu giá.".to_string(),
        },
        AskAITemplate {
            id: "market-timing-perfect".to_string(),
            title: "⏰ Thời điểm thị trường hoàn hảo - Khi nào vào/ra".to_string(),
            prompt: "Khi nào là thời điểm TUYỆT ĐỐI TỐT NHẤT để vào và thoát mã này? Phân tích: (1) Mô hình trong ngày và giờ tối ưu, (2) Chu kỳ hàng tuần và tính thời vụ hàng tháng, (3) Thời điểm luân chuyển ngành, (4) Mô hình tương quan thị trường, (5) Thời điểm tăng vọt khối lượng, (6) Tác động lịch tin tức/báo cáo thu nhập. Xác định các cửa sổ thời gian xác suất cao nhất cho lợi nhuận tối đa.".to_string(),
        },
        AskAITemplate {
            id: "breakout-analysis".to_string(),
            title: "🔥 Phân tích Breakout - Xác nhận Momentum & Khối lượng".to_string(),
            prompt: "Mã này có đang chuẩn bị cho một breakout hoặc breakdown lớn không? Phân tích: (1) Các mức kháng cử/hỗ trợ chính với giá chính xác, (2) Mô hình tích lũy khối lượng, (3) Tín hiệu xác nhận dòng tiền, (4) Xác suất hoàn thành mô hình biểu đồ, (5) Thời điểm catalyst và điều kiện thị trường, (6) Tỷ lệ rủi ro/lợi nhuận cho giao dịch breakout. Cung cấp mục tiêu breakout cụ thể và mức cắt lỗ.".to_string(),
        },
        AskAITemplate {
            id: "vpa-deep-dive".to_string(),
            title: "📈 VPA sâu - Bí mật Volume Price Action".to_string(),
            prompt: "Thực hiện phân tích Volume Price Action (VPA) sâu tiết lộ ý định thị trường ẩn giấu: (1) Bất thường mối quan hệ khối lượng vs giá, (2) Mô hình hoạt động của tiền chuyên nghiệp vs bán lẻ, (3) Giai đoạn tích lũy và phân phối, (4) Xác định đỉnh khối lượng, (5) Xác thực test hỗ trợ/kháng cự, (6) Dấu vết market maker. Giải mã những gì tiền thông minh thực sự đang làm.".to_string(),
        },
        AskAITemplate {
            id: "support-resistance-precision".to_string(),
            title: "🎯 Mức hỗ trợ & kháng cự chính xác".to_string(),
            prompt: "Xác định các mức hỗ trợ và kháng cự CHÍNH XÁC với độ chính xác toán học: (1) Điểm pivot lịch sử với xác nhận khối lượng, (2) Vùng hợp lưu Fibonacci, (3) Giao điểm đường MA, (4) Mức giá tâm lý, (5) Vùng volume profile, (6) Phá vỡ cấu trúc thị trường. Cung cấp giá vào/ra cụ thể với tỷ lệ thành công phần trăm.".to_string(),
        },

        // QUẢN LÝ RỦI RO & QUY MÔ VỊ THẾ
        AskAITemplate {
            id: "risk-management-complete".to_string(),
            title: "🛡️ Chiến lược quản lý rủi ro hoàn chỉnh".to_string(),
            prompt: "Thiết kế chiến lược quản lý rủi ro tối ưu cho mã này: (1) Quy mô vị thế sử dụng tiêu chí Kelly và volatility, (2) Đặt stop-loss sử dụng ATR và mức hỗ trợ, (3) Thang chốt lời với phần trăm cụ thể, (4) Tương quan portfolio và đa dạng hóa, (5) Giới hạn drawdown tối đa, (6) Giao thức thoát khẩn cấp. Đảm bảo bảo toàn vốn đồng thời tối đa hóa lợi nhuận.".to_string(),
        },

        // BỐI CẢNH THỊ TRƯỜNG & TƯƠNG QUAN
        AskAITemplate {
            id: "market-correlation".to_string(),
            title: "🌐 Tương quan thị trường & Phân tích luân chuyển ngành".to_string(),
            prompt: "Phân tích tương quan của mã này với chuyển động thị trường và luân chuyển ngành: (1) Tương quan VNINDEX và phân tích beta, (2) Vị trí lãnh đạo ngành, (3) Độ nhạy lãi suất, (4) Định vị chu kỳ kinh tế, (5) Tác động dòng vốn đầu tư nước ngoài, (6) Hiệu ứng tương quan tiền tệ. Xác định thời điểm thị trường tối ưu cho vào/ra.".to_string(),
        },

        // CƠ BẢN & ĐỊNH GIÁ
        AskAITemplate {
            id: "fundamental-deep-dive".to_string(),
            title: "📊 Phân tích cơ bản sâu".to_string(),
            prompt: "Thực hiện phân tích cơ bản toàn diện: (1) Phân tích tỷ số tài chính và xu hướng, (2) Tính bền vững tăng trưởng doanh thu và lợi nhuận, (3) Sức mạnh bảng cân đối và phân tích nợ, (4) Chất lượng và tạo ra dòng tiền, (5) Hiệu quả quản lý và quản trị, (6) Vị thế cạnh tranh và hào. Xác định giá trị nội tại và giá trị đầu tư.".to_string(),
        },
        AskAITemplate {
            id: "valuation-analysis".to_string(),
            title: "💎 Phân tích định giá - Ước tính giá trị hợp lý".to_string(),
            prompt: "Xác định giá trị hợp lý của mã này sử dụng nhiều phương pháp: (1) Phân tích DCF với các kịch bản tăng trưởng khác nhau, (2) Định giá tương đối vs đồng nghiệp, (3) Định giá dựa trên tài sản, (4) Đánh giá chất lượng thu nhập, (5) Phân loại tăng trưởng vs giá trị, (6) Tính toán margin of safety. Cung cấp phạm vi giá mục tiêu với khoảng tin cậy.".to_string(),
        },
    ]
}