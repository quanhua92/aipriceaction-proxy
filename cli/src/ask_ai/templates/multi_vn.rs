use crate::ask_ai::types::AskAITemplate;

pub fn get_multi_ticker_templates_vn() -> Vec<AskAITemplate> {
    vec![
        AskAITemplate {
            id: "portfolio-optimization".to_string(),
            title: "🎯 Chiến lược tối ưu hóa danh mục".to_string(),
            prompt: "Tối ưu hóa danh mục của tôi với những mã được chọn: (1) Phân tích tương quan và lợi ích đa dạng hóa, (2) Quy mô vị thế tối ưu cho mỗi mã, (3) Tối ưu hóa lợi nhuận điều chỉnh rủi ro, (4) Cân bằng phân bổ ngành, (5) Phối hợp thời điểm vào/ra, (6) Chiến lược hedging giữa các vị thế. Cung cấp phần trăm phân bổ cụ thể và quy tắc tái cân bằng.".to_string(),
        },
        AskAITemplate {
            id: "comparative-analysis".to_string(),
            title: "⚖️ Phân tích so sánh - Nên mua mã nào?".to_string(),
            prompt: "So sánh các mã này trên tất cả chiều kích: (1) Sức mạnh kỹ thuật và momentum, (2) Dòng tiền và sở thích tổ chức, (3) Định giá cơ bản và tăng trưởng, (4) Profile rủi ro-lợi nhuận, (5) Định vị ngành và luân chuyển, (6) Thời điểm catalyst và tiềm năng. Xếp hạng theo tính hấp dẫn đầu tư với lý do cụ thể.".to_string(),
        },
        AskAITemplate {
            id: "sector-rotation-play".to_string(),
            title: "🔄 Chiến lược luân chuyển ngành".to_string(),
            prompt: "Thiết kế chiến lược luân chuyển ngành sử dụng các mã này: (1) Phân tích vị trí chu kỳ ngành hiện tại, (2) Xác định ngành dẫn đầu vs tụt hậu, (3) Tín hiệu và trigger thời điểm luân chuyển, (4) Mô hình tương quan giữa các ngành, (5) Định vị chu kỳ kinh tế, (6) Chuỗi luân chuyển tối ưu. Định thời điểm chuyển đổi ngành cho alpha tối đa.".to_string(),
        },
        AskAITemplate {
            id: "pairs-trading".to_string(),
            title: "↔️ Cơ hội giao dịch cặp".to_string(),
            prompt: "Xác định cơ hội giao dịch cặp giữa các mã này: (1) Phân tích tương quan lịch sử, (2) Mô hình hồi quy trung bình, (3) Phân tích spread và giá trị hợp lý, (4) Tín hiệu phân kỳ momentum, (5) Quản lý rủi ro cho giao dịch cặp, (6) Thời điểm vào/ra tối ưu. Tìm cơ hội giá trị tương đối có lợi nhuận.".to_string(),
        },
        AskAITemplate {
            id: "risk-diversification".to_string(),
            title: "🛡️ Phân tích đa dạng hóa rủi ro".to_string(),
            prompt: "Phân tích đa dạng hóa rủi ro trên các mã này: (1) Ma trận tương quan và clustering, (2) Đa dạng hóa ngành và style, (3) Phân tích đóng góp volatility, (4) Đánh giá rủi ro đuôi, (5) Đánh giá rủi ro tập trung, (6) Tối ưu hóa tỷ lệ hedge. Xây dựng danh mục đa dạng hóa thực sự.".to_string(),
        },
        AskAITemplate {
            id: "momentum-basket".to_string(),
            title: "🚀 Chiến lược rổ momentum".to_string(),
            prompt: "Tạo chiến lược rổ momentum: (1) Xếp hạng và chấm điểm momentum, (2) Quy tắc luân chuyển trong rổ, (3) Tiêu chí thêm/bớt, (4) Quy mô vị thế theo sức mạnh momentum, (5) Quản lý rủi ro cho chiến lược momentum, (6) Phát hiện suy giảm momentum. Cùng nhau theo đuổi những xu hướng mạnh nhất.".to_string(),
        },
    ]
}