use crate::ask_ai::types::AskAITemplate;

pub fn get_money_flow_templates_vn() -> Vec<AskAITemplate> {
    vec![
        AskAITemplate {
            id: "market-money-flow-analysis".to_string(),
            title: "🌊 Phân tích dòng tiền thị trường toàn diện".to_string(),
            prompt: "Phân tích các mô hình dòng tiền toàn thị trường: (1) Luân chuyển ngành dựa trên xếp hạng dòng tiền, (2) Sự phân kỳ giữa tiền thông minh vs tâm lý bán lẻ, (3) Mô hình tích lũy/phân phối tổ chức, (4) Tương quan dòng tiền giữa các ngành, (5) Thay đổi leadership thị trường và hàm ý, (6) Phân kỳ dòng tiền với price action. Xác định nơi tiền thông minh đang định vị để tối đa hóa lợi nhuận.".to_string(),
        },
        AskAITemplate {
            id: "sector-money-flow".to_string(),
            title: "🏭 Dòng tiền ngành & Chiến lược luân chuyển".to_string(),
            prompt: "Thiết kế chiến lược luân chuyển ngành dựa trên dòng tiền: (1) Xếp hạng và xu hướng dòng tiền ngành hiện tại, (2) Xác định ngành dẫn đầu vs tụt hậu, (3) Momentum và gia tốc dòng tiền, (4) Tương quan ngành với market leaders, (5) Định vị chu kỳ kinh tế thông qua dòng tiền, (6) Tín hiệu chuyển đổi ngành tối ưu. Định thời điểm luân chuyển ngành sử dụng chuyển động tiền thông minh.".to_string(),
        },
        AskAITemplate {
            id: "smart-money-tracking".to_string(),
            title: "🧠 Theo dõi & Đi theo tiền thông minh".to_string(),
            prompt: "Theo dõi và đi theo chuyển động tiền thông minh: (1) Xác định cổ phiếu có mua tổ chức mạnh nhất, (2) Phân tích phân kỳ dòng tiền vs giá, (3) Volume profile và điểm vào của tiền thông minh, (4) Xác định giai đoạn tích lũy, (5) Tín hiệu cảnh báo phân phối, (6) Chiến lược thoát của tiền thông minh. Theo các tổ chức để có lợi nhuận vượt trội.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-divergence".to_string(),
            title: "📈 Phân tích phân kỳ dòng tiền".to_string(),
            prompt: "Phân tích các phân kỳ dòng tiền để tìm cơ hội: (1) Xác định phân kỳ giá vs dòng tiền, (2) Mô hình phân kỳ tăng và giảm, (3) Phân kỳ ẩn và tín hiệu tiếp tục, (4) Sức mạnh và độ tin cậy của phân kỳ, (5) Khung thời gian để giải quyết phân kỳ, (6) Chiến lược giao dịch cho mỗi loại phân kỳ. Khai thác sự kém hiệu quả của thị trường.".to_string(),
        },
        AskAITemplate {
            id: "market-leaders-analysis".to_string(),
            title: "👑 Phân tích dòng tiền Market Leaders".to_string(),
            prompt: "Phân tích market leaders qua góc độ dòng tiền: (1) Leadership thị trường hiện tại dựa trên dòng tiền, (2) Mô hình và thời điểm luân chuyển leadership, (3) Phân tích chất lượng vs số lượng dòng tiền, (4) Leadership bền vững vs tạm thời, (5) Tương quan giữa leaders và hướng thị trường, (6) Xác định market leaders tiềm năng tiếp theo. Xác định và theo đuổi những xu hướng mạnh nhất.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-momentum".to_string(),
            title: "⚡ Chiến lược momentum dòng tiền".to_string(),
            prompt: "Phát triển chiến lược momentum dòng tiền: (1) Tín hiệu gia tốc và giảm tốc dòng tiền, (2) Hệ thống xếp hạng và chấm điểm momentum, (3) Quy tắc vào và ra dựa trên flow momentum, (4) Quản lý rủi ro cho chiến lược momentum, (5) Tín hiệu cảnh báo phân kỳ momentum, (6) Xây dựng danh mục sử dụng flow momentum. Xây dựng phương pháp momentum có hệ thống.".to_string(),
        },
        AskAITemplate {
            id: "institutional-vs-retail".to_string(),
            title: "🏛️ Dòng tiền tổ chức vs bán lẻ".to_string(),
            prompt: "Phân tích các mô hình dòng tiền tổ chức vs bán lẻ: (1) Sự phân kỳ giữa tiền thông minh vs tâm lý bán lẻ, (2) Tích lũy tổ chức trong hoảng loạn bán lẻ, (3) Phân phối trong hưng phấn bán lẻ, (4) Phân tích khối lượng cho hoạt động tổ chức vs bán lẻ, (5) Cơ hội contrarian từ phân kỳ dòng chảy, (6) Định thời điểm thị trường sử dụng dòng tổ chức. Định vị cùng với tiền thông minh.".to_string(),
        },
    ]
}